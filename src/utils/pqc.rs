//! NIST post-quantum message sealing (server-side, SSR/Tauri only).
//!
//! Hybrid construction:
//! - ML-KEM-1024 (FIPS 203) encapsulates a 32-byte shared secret per message
//! - ChaCha20-Poly1305 encrypts the zstd-compressed payload with that secret
//! - ML-DSA-65 (FIPS 204) signs everything except the signature field
//!
//! Sealed blob layout:
//! ```text
//! magic "PQM1" (4) | version u8 | suite u8
//! u16 kem_ct_len | u16 sig_len | u32 ct_len   (little-endian)
//! nonce[12]
//! kem_ct[kem_ct_len]
//! sig[sig_len]
//! ciphertext[ct_len]
//! ```
//! The signature covers header + nonce + kem_ct + ciphertext.

use ml_dsa::{Generate, Keypair, MlDsa65, Signer, SigningKey, Verifier, VerifyingKey};
use ml_kem::kem::{self, Decapsulate, Encapsulate, FromSeed};
use ml_kem::{Kem, MlKem1024};
use serde::{Deserialize, Serialize};

const MAGIC: &[u8; 4] = b"PQM1";
const VERSION: u8 = 1;
const SUITE_MLKEM1024_MLDSA65: u8 = 1;
const HEADER_LEN: usize = 4 + 1 + 1 + 2 + 2 + 4;
const NONCE_LEN: usize = 12;

type KemDecap = ml_kem::DecapsulationKey<MlKem1024>;
type KemEncap = ml_kem::EncapsulationKey<MlKem1024>;

/// Server-held PQC key material. Only the two 32-byte seeds are persisted;
/// all key material is deterministically re-derived from them.
#[derive(Clone)]
pub struct PqcKeys {
    kem_dk: KemDecap,
    kem_ek: KemEncap,
    dsa_sk: SigningKey<MlDsa65>,
    dsa_vk: VerifyingKey<MlDsa65>,
}

/// Serialized form persisted alongside the message store: two 32-byte seeds.
#[derive(Serialize, Deserialize)]
pub struct PqcKeyfile {
    kem_seed: Vec<u8>,
    dsa_seed: Vec<u8>,
}

fn pqc_err(msg: impl Into<String>) -> Box<dyn std::error::Error> {
    msg.into().into()
}

fn seed_from_bytes(bytes: &[u8]) -> Result<kem::Seed<MlKem1024>, Box<dyn std::error::Error>> {
    kem::Seed::<MlKem1024>::try_from(bytes)
        .map_err(|_| pqc_err("KEM seed must be 32 bytes"))
}

fn dsa_seed_from_bytes(bytes: &[u8]) -> Result<ml_dsa::Seed, Box<dyn std::error::Error>> {
    ml_dsa::Seed::try_from(bytes).map_err(|_| pqc_err("DSA seed must be 32 bytes"))
}

impl PqcKeys {
    /// Generate a fresh ML-KEM-1024 + ML-DSA-65 keypair set.
    pub fn generate() -> Result<Self, Box<dyn std::error::Error>> {
        let (kem_dk, kem_ek) = MlKem1024::generate_keypair();
        let dsa_sk = SigningKey::<MlDsa65>::generate();
        let dsa_vk = dsa_sk.verifying_key();
        Ok(Self {
            kem_dk,
            kem_ek,
            dsa_sk,
            dsa_vk,
        })
    }

    /// Rebuild keys from their persisted seeds.
    pub fn from_keyfile(kf: &PqcKeyfile) -> Result<Self, Box<dyn std::error::Error>> {
        let kem_seed = seed_from_bytes(&kf.kem_seed)?;
        let (kem_dk, kem_ek) = MlKem1024::from_seed(&kem_seed);
        let dsa_seed = dsa_seed_from_bytes(&kf.dsa_seed)?;
        let dsa_sk = SigningKey::<MlDsa65>::from_seed(&dsa_seed);
        let dsa_vk = dsa_sk.verifying_key();
        Ok(Self {
            kem_dk,
            kem_ek,
            dsa_sk,
            dsa_vk,
        })
    }

    /// Serialize the two seeds for persistence.
    pub fn to_keyfile(&self) -> PqcKeyfile {
        let kem_seed = self
            .kem_dk
            .to_seed()
            .ok_or_else(|| pqc_err("KEM decapsulation key has no seed"))
            .unwrap_or_default();
        PqcKeyfile {
            kem_seed: kem_seed.as_slice().to_vec(),
            dsa_seed: self.dsa_sk.to_seed().as_slice().to_vec(),
        }
    }

    /// PQC-seal a plaintext payload. Returns the binary blob described above.
    pub fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 1. ML-KEM-1024 encapsulation → 32-byte shared secret + ciphertext
        let (kem_ct, shared_secret) = self.kem_ek.encapsulate();
        let kem_ct_bytes: &[u8] = kem_ct.as_ref();

        // 2. Shared secret → ChaCha20-Poly1305 key; compress + encrypt payload
        let compressed = zstd::encode_all(plaintext, 3)?;
        let key = super::crypto::format_key(shared_secret.as_ref());
        let mut nonce_bytes = [0u8; NONCE_LEN];
        {
            use rand::RngCore;
            rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        }
        let ct_bytes = aead_seal(&compressed, &key, &nonce_bytes)?;

        // 3. Assemble signed region: header + nonce + kem_ct + ciphertext
        let kem_ct_len = kem_ct_bytes.len() as u16;
        let ct_len = ct_bytes.len() as u32;
        let mut prefix = Vec::with_capacity(HEADER_LEN + NONCE_LEN + kem_ct_bytes.len());
        prefix.extend_from_slice(MAGIC);
        prefix.push(VERSION);
        prefix.push(SUITE_MLKEM1024_MLDSA65);
        prefix.extend_from_slice(&kem_ct_len.to_le_bytes());
        prefix.extend_from_slice(&0u16.to_le_bytes()); // sig len patched below
        prefix.extend_from_slice(&ct_len.to_le_bytes());
        prefix.extend_from_slice(&nonce_bytes);
        prefix.extend_from_slice(kem_ct_bytes);

        let mut signed_region = prefix.clone();
        signed_region.extend_from_slice(&ct_bytes);
        let sig = self.dsa_sk.sign(&signed_region);
        let sig_bytes = sig.encode();

        // 4. Final blob: prefix (sig len patched) + sig + ciphertext
        let sig_len = sig_bytes.as_slice().len() as u16;
        prefix[8..10].copy_from_slice(&sig_len.to_le_bytes());
        let mut blob = prefix;
        blob.extend_from_slice(sig_bytes.as_slice());
        blob.extend_from_slice(&ct_bytes);
        Ok(blob)
    }

    /// Open a blob produced by `seal`. Verifies the ML-DSA signature before
    /// decapsulating — tampered blobs are rejected.
    pub fn open(&self, blob: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if blob.len() < HEADER_LEN + NONCE_LEN {
            return Err(pqc_err("sealed blob too short"));
        }
        if &blob[0..4] != MAGIC {
            return Err(pqc_err("bad magic — not a PQC-sealed payload"));
        }
        if blob[4] != VERSION {
            return Err(pqc_err(format!("unsupported version {}", blob[4])));
        }
        if blob[5] != SUITE_MLKEM1024_MLDSA65 {
            return Err(pqc_err(format!("unsupported suite {}", blob[5])));
        }

        let kem_ct_len = u16::from_le_bytes([blob[6], blob[7]]) as usize;
        let sig_len = u16::from_le_bytes([blob[8], blob[9]]) as usize;
        let ct_len = u32::from_le_bytes([blob[10], blob[11], blob[12], blob[13]]) as usize;

        let nonce_off = HEADER_LEN;
        let kem_ct_off = nonce_off + NONCE_LEN;
        let sig_off = kem_ct_off + kem_ct_len;
        let ct_off = sig_off + sig_len;
        let end = ct_off + ct_len;
        if end > blob.len() || kem_ct_len == 0 || sig_len == 0 || ct_len == 0 {
            return Err(pqc_err("sealed blob truncated or malformed"));
        }

        let kem_ct_bytes = &blob[kem_ct_off..sig_off];
        let sig_bytes = &blob[sig_off..ct_off];
        let ct_bytes = &blob[ct_off..end];
        let nonce_bytes = &blob[nonce_off..kem_ct_off];

        // Reconstruct the signed region: everything except the sig field.
        let mut signed_region = Vec::with_capacity(end - sig_len);
        signed_region.extend_from_slice(&blob[..sig_off]);
        signed_region.extend_from_slice(ct_bytes);

        let sig = ml_dsa::Signature::<MlDsa65>::try_from(sig_bytes)
            .map_err(|_| pqc_err("bad ML-DSA signature encoding"))?;
        self.dsa_vk
            .verify(&signed_region, &sig)
            .map_err(|_| pqc_err("ML-DSA signature verification failed"))?;

        // ML-KEM decapsulation → shared secret → AEAD open → decompress
        let shared_secret = self
            .kem_dk
            .decapsulate_slice(kem_ct_bytes)
            .map_err(|e| pqc_err(format!("ML-KEM decapsulation failed: {:?}", e)))?;
        let key = super::crypto::format_key(shared_secret.as_ref());
        let compressed = aead_open(ct_bytes, &key, nonce_bytes)?;
        let plain = zstd::decode_all(&compressed[..])?;
        Ok(plain)
    }
}

fn aead_seal(
    plaintext: &[u8],
    key: &[u8; 32],
    nonce_bytes: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use chacha20poly1305::{
        aead::{Aead, KeyInit},
        ChaCha20Poly1305, Nonce,
    };
    let cipher = ChaCha20Poly1305::new_from_slice(key).expect("32-byte key");
    let nonce = Nonce::from_slice(nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| pqc_err(format!("aead encrypt: {:?}", e)))?;
    Ok(ciphertext)
}

fn aead_open(
    data: &[u8],
    key: &[u8; 32],
    nonce_bytes: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use chacha20poly1305::{
        aead::{Aead, KeyInit},
        ChaCha20Poly1305, Nonce,
    };
    if nonce_bytes.len() != NONCE_LEN {
        return Err(pqc_err("bad nonce length"));
    }
    let cipher = ChaCha20Poly1305::new_from_slice(key).expect("32-byte key");
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, data)
        .map_err(|e| pqc_err(format!("aead decrypt: {:?}", e)))
        .map(|v| v.to_vec())
}
