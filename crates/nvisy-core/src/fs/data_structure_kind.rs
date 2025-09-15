//! Data structure type classification
//!
//! This module provides classification for different ways data can be structured,
//! from highly organized formats to completely unstructured content.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};

use crate::fs::DataSensitivity;

/// Classification of data based on its structural organization
///
/// This enum distinguishes between different levels of data organization,
/// from highly structured formats with defined schemas to completely
/// unstructured content without predefined organization.
///
/// # Examples
///
/// ```rust
/// use nvisy_core::DataStructureKind;
///
/// let structured = DataStructureKind::HighlyStructured;
/// assert_eq!(structured.name(), "Highly Structured");
/// assert!(structured.has_schema());
///
/// let unstructured = DataStructureKind::Unstructured;
/// assert!(!unstructured.has_schema());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(EnumIter, EnumString)]
pub enum DataStructureKind {
    /// Highly Structured Data
    ///
    /// Data with rigid schema, defined relationships, and strict formatting rules.
    /// Examples: Relational database tables, XML with XSD schema, JSON with JSON Schema.
    ///
    /// **Schema**: Required and enforced
    /// **Queryable**: Highly queryable with structured query languages
    /// **Parsing**: Predictable parsing with validation
    HighlyStructured,

    /// Semi-Structured Data
    ///
    /// Data with some organizational structure but flexible schema.
    /// Examples: JSON without strict schema, XML without XSD, CSV files, log files.
    ///
    /// **Schema**: Optional or loosely defined
    /// **Queryable**: Moderately queryable with specialized tools
    /// **Parsing**: Parseable but may require schema inference
    SemiStructured,

    /// Unstructured Data
    ///
    /// Data without predefined format, schema, or organizational structure.
    /// Examples: Plain text, images, audio, video, documents, emails.
    ///
    /// **Schema**: No schema
    /// **Queryable**: Requires full-text search or content analysis
    /// **Parsing**: Content-dependent parsing and analysis
    Unstructured,
}

impl DataStructureKind {
    /// Get the base sensitivity level for this structure type
    ///
    /// Note: Actual sensitivity depends on the content, not just the structure
    pub fn base_sensitivity_level(&self) -> DataSensitivity {
        match self {
            // Structure type alone doesn't determine sensitivity
            // Content analysis is required for actual sensitivity assessment
            DataStructureKind::HighlyStructured
            | DataStructureKind::SemiStructured
            | DataStructureKind::Unstructured => DataSensitivity::Low,
        }
    }

    /// Check if this structure type has a defined schema
    pub fn has_schema(&self) -> bool {
        matches!(self, DataStructureKind::HighlyStructured)
    }

    /// Check if this structure type is easily queryable
    pub fn is_queryable(&self) -> bool {
        !matches!(self, DataStructureKind::Unstructured)
    }

    /// Check if parsing is predictable for this structure type
    pub fn has_predictable_parsing(&self) -> bool {
        matches!(self, DataStructureKind::HighlyStructured)
    }

    /// Check if this structure type supports relationship queries
    pub fn supports_relationships(&self) -> bool {
        matches!(self, DataStructureKind::HighlyStructured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structure_characteristics() {
        let highly_structured = DataStructureKind::HighlyStructured;
        assert!(highly_structured.has_schema());
        assert!(highly_structured.is_queryable());
        assert!(highly_structured.has_predictable_parsing());

        let unstructured = DataStructureKind::Unstructured;
        assert!(!unstructured.has_schema());
        assert!(!unstructured.is_queryable());
        assert!(!unstructured.has_predictable_parsing());

        let highly_structured = DataStructureKind::HighlyStructured;
        assert!(highly_structured.supports_relationships());
        assert!(highly_structured.has_schema());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialization() {
        let structure_type = DataStructureKind::SemiStructured;
        let json = serde_json::to_string(&structure_type).unwrap();
        let deserialized: DataStructureKind = serde_json::from_str(&json).unwrap();
        assert_eq!(structure_type, deserialized);
    }
}
