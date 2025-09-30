//! Data sensitivity level classification
//!
//! This module provides a systematic way to classify data based on sensitivity
//! and risk levels for proper handling and compliance requirements.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

/// Data sensitivity levels for risk assessment and handling requirements
///
/// This enum provides a hierarchical classification system for data sensitivity,
/// allowing for proper risk assessment and appropriate security controls.
///
/// The levels are ordered from lowest to highest sensitivity:
/// `None < Low < Medium < High`
///
/// # Examples
///
/// ```rust
/// use nvisy_core::DataSensitivity;
///
/// let high = DataSensitivity::High;
/// let medium = DataSensitivity::Medium;
/// let low = DataSensitivity::Low;
///
/// assert!(high > medium);
/// assert!(medium > low);
/// assert!(high.requires_special_handling());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(EnumIter, EnumString, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataSensitivity {
    /// No sensitivity - public or non-sensitive data
    ///
    /// Data that can be freely shared without privacy or security concerns.
    /// Examples: Public documentation, marketing materials, published research.
    None = 0,

    /// Low sensitivity - internal or limited distribution
    ///
    /// Data with minimal privacy implications, typically internal business data.
    /// Examples: General business metrics, non-personal analytics, public contact info.
    Low = 1,

    /// Medium sensitivity - requires basic protection
    ///
    /// Data that could cause minor harm if exposed inappropriately.
    /// Examples: Internal communications, aggregated demographics, business contacts.
    Medium = 2,

    /// High sensitivity - requires maximum protection
    ///
    /// Data that could cause severe harm, legal liability, or regulatory violations if exposed.
    /// Examples: Financial data, health records, biometric data, government IDs, personal contact information.
    High = 3,
}

impl DataSensitivity {
    /// Get the numeric value of this sensitivity level (0-3)
    pub fn level(&self) -> u8 {
        *self as u8
    }

    /// Check if this sensitivity level requires special handling
    pub fn requires_special_handling(&self) -> bool {
        *self >= DataSensitivity::High
    }

    /// Check if this sensitivity level requires encryption
    pub fn requires_encryption(&self) -> bool {
        *self >= DataSensitivity::Medium
    }

    /// Check if this sensitivity level requires access logging
    pub fn requires_access_logging(&self) -> bool {
        *self >= DataSensitivity::High
    }

    /// Check if this sensitivity level requires data retention policies
    pub fn requires_retention_policy(&self) -> bool {
        *self >= DataSensitivity::Medium
    }

    /// Check if this sensitivity level requires regulatory compliance oversight
    pub fn requires_compliance_oversight(&self) -> bool {
        *self >= DataSensitivity::High
    }

    /// Get the recommended maximum retention period in days (None = indefinite)
    pub fn max_retention_days(&self) -> Option<u32> {
        match self {
            DataSensitivity::None => None,         // Indefinite
            DataSensitivity::Low => Some(2555),    // ~7 years
            DataSensitivity::Medium => Some(1095), // 3 years
            DataSensitivity::High => Some(90),     // 90 days
        }
    }

    /// Get all sensitivity levels in ascending order
    pub fn all() -> Vec<DataSensitivity> {
        vec![
            DataSensitivity::None,
            DataSensitivity::Low,
            DataSensitivity::Medium,
            DataSensitivity::High,
        ]
    }

    /// Create from a numeric level (0-3)
    pub fn from_level(level: u8) -> Option<DataSensitivity> {
        match level {
            0 => Some(DataSensitivity::None),
            1 => Some(DataSensitivity::Low),
            2 => Some(DataSensitivity::Medium),
            3 => Some(DataSensitivity::High),
            _ => None,
        }
    }
}

impl PartialOrd for DataSensitivity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DataSensitivity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordering() {
        assert!(DataSensitivity::High > DataSensitivity::Medium);
        assert!(DataSensitivity::Medium > DataSensitivity::Low);
        assert!(DataSensitivity::Low > DataSensitivity::None);
    }

    #[test]
    fn test_levels() {
        assert_eq!(DataSensitivity::None.level(), 0);
        assert_eq!(DataSensitivity::Low.level(), 1);
        assert_eq!(DataSensitivity::Medium.level(), 2);
        assert_eq!(DataSensitivity::High.level(), 3);
    }

    #[test]
    fn test_from_level() {
        assert_eq!(DataSensitivity::from_level(0), Some(DataSensitivity::None));
        assert_eq!(DataSensitivity::from_level(4), None);
    }

    #[test]
    fn test_requirements() {
        let none = DataSensitivity::None;
        let low = DataSensitivity::Low;
        let medium = DataSensitivity::Medium;
        let high = DataSensitivity::High;
        // Special handling
        assert!(!none.requires_special_handling());
        assert!(!low.requires_special_handling());
        assert!(!medium.requires_special_handling());
        assert!(high.requires_special_handling());

        // Encryption
        assert!(!none.requires_encryption());
        assert!(!low.requires_encryption());
        assert!(medium.requires_encryption());
        assert!(high.requires_encryption());

        // Access logging
        assert!(!none.requires_access_logging());
        assert!(!low.requires_access_logging());
        assert!(!medium.requires_access_logging());
        assert!(high.requires_access_logging());

        // Compliance oversight
        assert!(!none.requires_compliance_oversight());
        assert!(!low.requires_compliance_oversight());
        assert!(!medium.requires_compliance_oversight());
        assert!(high.requires_compliance_oversight());
    }

    #[test]
    fn test_retention_periods() {
        assert_eq!(DataSensitivity::None.max_retention_days(), None);
        assert_eq!(DataSensitivity::Low.max_retention_days(), Some(2555));
        assert_eq!(DataSensitivity::Medium.max_retention_days(), Some(1095));
        assert_eq!(DataSensitivity::High.max_retention_days(), Some(90));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DataSensitivity::High), "High");
        assert_eq!(format!("{}", DataSensitivity::None), "None");
    }

    #[test]
    fn test_all_levels() {
        let all = DataSensitivity::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], DataSensitivity::None);
        assert_eq!(all[3], DataSensitivity::High);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialization() {
        let level = DataSensitivity::High;
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: DataSensitivity = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }
}
