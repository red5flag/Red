# Red — Tauri 2 Migration Map

## Workspace Audit Summary

### Current Structure
- **Package name**: `farley` (binary + cdylib/rlib)
- **Framework**: Leptos 0.8 with SSR (Axum) + hydrate (WASM) + CSR modes
- **Feature flags**: `default=[]`, `hydrate`, `ssr`, `csr`
- **Build tool**: cargo-leptos (`watch.sh` → `cargo leptos watch`)
- **Server entry**: `src/main.rs` (Axum router with API routes + Leptos routes)
- **Client entry**: `src/lib.rs` (`#[wasm_bindgen(start)] hydrate()`)
- **CSR entry**: `src/main.rs` else-branch → `mount_to_body(App)`

### Module Layout
```
src/
  api/          — email validation API (SSR)
  app.rs        — App component, shell, store initialization
  components/   — calendar, navbar, tabs, footer, messenger, search, rule_engine
  models/       — domain models (portfolio, transaction, channel, messenger, booking, etc.)
  pages/        — UI pages (login, overview, portfolios, transactions, calendar, settings, etc.)
  server.rs     — server functions (OpenMaps autocomplete, save portfolio, PDF conversion)
  storage/      — sled DataStore (SSR-only) + RocksDB store (unused, no rocksdb in Cargo.toml)
  stores/       — Leptos reactive stores (app, calendar, messenger, notifications, org, portfolio, etc.)
  types/        — shared enums (UserRole, Currency, AssetType, TransactionType, etc.)
  utils/        — crypto (ChaCha20-Poly1305 + zstd), DOM helpers
```

### Key Findings
1. **sled**: Used in `src/storage/db.rs` — flat keyspace (`uuid` for portfolios, `msg:uuid` for messages, `cred:username` for credentials). DashMap write-through cache. SSR-only.
2. **RocksDB**: `src/storage/rocks_db.rs` exists but `rocksdb` is NOT in Cargo.toml — dead code, should be removed.
3. **Crypto**: ChaCha20-Poly1305 + zstd compression in `src/utils/crypto.rs`. SSR-only for encrypt/decrypt; hydrate has localStorage stubs.
4. **Networking**: `reqwest` (SSR) for OpenMaps, `gloo-net` (hydrate) for HTTP. Server functions use `#[server(...)]` macro.
5. **web-sys**: Extensive features for camera/media (camscan), drag-drop, touch, canvas, file I/O.
6. **PDF**: `printpdf` (SSR-only) for image→PDF conversion in `server.rs`.
7. **Auth**: bcrypt, jsonwebtoken, argon2, totp-rs, chacha20poly1305 — all SSR-only.
8. **Server functions**: 3 in `server.rs` (OpenMaps, SavePortfolio, ConvertImagesToPdf), auth API routes in `main.rs`.
9. **No `public/` dir** — referenced in leptos config but doesn't exist.
10. **No tests** currently.

### Migration Plan

#### Milestone 1: Tauri 2 Skeleton + Android Path
- Add `tauri` feature to Cargo.toml
- Create `src-tauri/` with Tauri 2 config
- Configure Android + desktop targets
- Leptos CSR mode launches inside Tauri WebView
- Install tauri-cli

#### Milestone 2: red-core + Typed IPC
- Create `crates/red-core/` with shared domain models
- Move types/ and models/ into red-core
- Define typed IPC request/response structs
- Tauri capabilities/security config

#### Milestone 3: red-storage + Schema Versioning
- Create `crates/red-storage/` with sled service
- Design proper tree/prefix layout with ordered keys
- Implement schema versioning + migration runner
- Move storage/db.rs logic into red-storage

#### Milestone 4: Native Asset Queries
- Tauri commands for paginated/filtered asset queries
- Cursor-based pagination

#### Milestone 5: Native Channel/Message Service
- Tauri commands for channel/message CRUD
- Ordered compound keys for efficient message retrieval

#### Milestone 6: Native Networking
- Move reqwest/gloo-net logic behind Tauri commands
- WebSocket/realtime, reconnect, retry/backoff

#### Milestone 7: Native Document Processing
- Move PDF generation, compression, hashing to native
- Filesystem-backed document storage

#### Milestone 8: Transaction Service
- Client-side validation, pending queue, idempotency
- Audit metadata, retries

#### Milestone 9: UI Virtualization + IPC Optimization
- Virtualized message/asset lists
- Batched IPC, minimal DOM updates

#### Milestone 10: Profiling, Benchmarks, Release
- Criterion benchmarks for sled workloads
- Android release validation
- Final cleanup
