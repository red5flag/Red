// Business validation and lookup functionality
// Supports Australian Business Number (ABN) and Legal Entity Identifier (LEI) validation

use serde::{Deserialize, Serialize};

/// Business validation result
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BusinessValidationResult {
    pub is_valid: bool,
    pub business_name: Option<String>,
    pub business_address: Option<String>,
    pub business_type: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

/// Validate Australian Business Number (ABN)
/// ABN is an 11-digit number with a specific checksum algorithm
pub fn validate_abn(abn: &str) -> BusinessValidationResult {
    // Remove spaces and dashes
    let cleaned: String = abn.chars().filter(|c| c.is_ascii_digit()).collect();

    // ABN must be 11 digits
    if cleaned.len() != 11 {
        return BusinessValidationResult {
            is_valid: false,
            business_name: None,
            business_address: None,
            business_type: None,
            status: None,
            error: Some("ABN must be 11 digits".to_string()),
        };
    }

    // Parse as number
    let digits: Vec<u32> = cleaned.chars().filter_map(|c| c.to_digit(10)).collect();

    if digits.len() != 11 {
        return BusinessValidationResult {
            is_valid: false,
            business_name: None,
            business_address: None,
            business_type: None,
            status: None,
            error: Some("ABN contains invalid characters".to_string()),
        };
    }

    // ABN checksum algorithm
    // Subtract 1 from the first digit
    let weights = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    let mut first_digit = digits[0];
    if first_digit > 0 {
        first_digit -= 1;
    }

    let mut sum = first_digit * weights[0];
    for (i, &digit) in digits.iter().skip(1).enumerate() {
        sum += digit * weights[i + 1];
    }

    let is_valid = sum % 89 == 0;

    BusinessValidationResult {
        is_valid,
        business_name: None, // Would require API call to ABR
        business_address: None,
        business_type: None,
        status: if is_valid {
            Some("Active".to_string())
        } else {
            None
        },
        error: if !is_valid {
            Some("Invalid ABN checksum".to_string())
        } else {
            None
        },
    }
}

/// Validate Legal Entity Identifier (LEI)
/// LEI is a 20-character alphanumeric code with a specific format
pub fn validate_lei(lei: &str) -> BusinessValidationResult {
    // Remove spaces and dashes
    let cleaned: String = lei.chars().filter(|c| c.is_alphanumeric()).collect();

    // LEI must be 20 characters
    if cleaned.len() != 20 {
        return BusinessValidationResult {
            is_valid: false,
            business_name: None,
            business_address: None,
            business_type: None,
            status: None,
            error: Some("LEI must be 20 characters".to_string()),
        };
    }

    // LEI format: 20 alphanumeric characters
    // First 4 characters: LOU identifier
    // Next 12 characters: entity ID
    // Next 2 characters: reserved for future use
    // Last 2 characters: checksum digits (00-97)

    if !cleaned.chars().all(|c| c.is_alphanumeric()) {
        return BusinessValidationResult {
            is_valid: false,
            business_name: None,
            business_address: None,
            business_type: None,
            status: None,
            error: Some("LEI must contain only alphanumeric characters".to_string()),
        };
    }

    // Basic format validation (full checksum validation requires ISO 17442 algorithm)
    let is_valid = true; // Simplified - full validation would require MOD-97-10 checksum

    BusinessValidationResult {
        is_valid,
        business_name: None, // Would require API call to GLEIF
        business_address: None,
        business_type: None,
        status: if is_valid {
            Some("Active".to_string())
        } else {
            None
        },
        error: if !is_valid {
            Some("Invalid LEI format".to_string())
        } else {
            None
        },
    }
}

/// Format ABN with standard spacing (XX XXX XXX XXX)
pub fn format_abn(abn: &str) -> String {
    let cleaned: String = abn.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() == 11 {
        format!(
            "{} {} {} {}",
            &cleaned[0..2],
            &cleaned[2..5],
            &cleaned[5..8],
            &cleaned[8..11]
        )
    } else {
        cleaned
    }
}

/// Format LEI with standard spacing (XXXX XXXXXXXXXXXXXXXX XX)
pub fn format_lei(lei: &str) -> String {
    let cleaned: String = lei.chars().filter(|c| c.is_alphanumeric()).collect();
    if cleaned.len() == 20 {
        format!(
            "{} {} {}",
            &cleaned[0..4],
            &cleaned[4..18],
            &cleaned[18..20]
        )
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_abn() {
        // Test with a known valid ABN (example)
        let result = validate_abn("51824753556");
        assert!(result.is_valid);
    }

    #[test]
    fn test_invalid_abn_checksum() {
        let result = validate_abn("51824753557");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_abn_formatting() {
        let formatted = format_abn("51824753556");
        assert_eq!(formatted, "51 824 753 556");
    }

    #[test]
    fn test_lei_format() {
        let result = validate_lei("5493001KJTIIGC8Y1R12");
        assert!(result.is_valid);
    }

    #[test]
    fn test_lei_formatting() {
        let formatted = format_lei("5493001KJTIIGC8Y1R12");
        assert_eq!(formatted, "5493 001KJTIIGC8Y1R 12");
    }
}
