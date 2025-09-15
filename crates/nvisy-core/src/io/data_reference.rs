//! Data reference definitions
//!
//! This module provides the DataReference struct for referencing and
//! tracking content within the Nvisy system.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::io::Content;

/// Reference to data with source tracking and content information
///
/// A `DataReference` provides a lightweight way to reference data content
/// while maintaining information about its source location and optional
/// mapping within that source.
///
/// # Examples
///
/// ```rust
/// use nvisy_core::{DataReference, Content};
///
/// let content = Content::Text("Hello, world!".to_string());
/// let data_ref = DataReference::new(content)
///     .with_mapping_id("line-42");
///
/// assert!(data_ref.mapping_id().is_some());
/// assert_eq!(data_ref.mapping_id().unwrap(), "line-42");
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataReference {
    /// Unique identifier for the source containing this data
    /// Using UUID v7 for time-ordered, globally unique identification
    source_id: Uuid,

    /// Optional identifier that defines the position/location of the data within the source
    /// Examples: line numbers, byte offsets, element IDs, XPath expressions
    mapping_id: Option<String>,

    /// The actual content data
    content_type: Content,
}

impl DataReference {
    /// Create a new data reference with auto-generated source ID
    pub fn new(content: Content) -> Self {
        Self {
            source_id: Uuid::new_v4(),
            mapping_id: None,
            content_type: content,
        }
    }

    /// Create a new data reference with specific source ID
    pub fn with_source_id(source_id: Uuid, content: Content) -> Self {
        Self {
            source_id,
            mapping_id: None,
            content_type: content,
        }
    }

    /// Set the mapping ID for this data reference
    pub fn with_mapping_id<S: Into<String>>(mut self, mapping_id: S) -> Self {
        self.mapping_id = Some(mapping_id.into());
        self
    }

    /// Get the source ID
    pub fn source_id(&self) -> Uuid {
        self.source_id
    }

    /// Get the mapping ID, if any
    pub fn mapping_id(&self) -> Option<&str> {
        self.mapping_id.as_deref()
    }

    /// Get a reference to the content
    pub fn content(&self) -> &Content {
        &self.content_type
    }

    /// Get the content type name
    pub fn content_type_name(&self) -> &'static str {
        self.content_type.type_name()
    }

    /// Get the estimated size of the content in bytes
    pub fn estimated_size(&self) -> usize {
        self.content_type.estimated_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_reference_creation() {
        let content = Content::text("Hello, world!");
        let data_ref = DataReference::new(content);

        assert_eq!(data_ref.content_type_name(), "text");
        assert!(data_ref.mapping_id().is_none());
        assert_eq!(data_ref.estimated_size(), 13);
    }

    #[test]
    fn test_data_reference_with_mapping() {
        let content = Content::text("Test content");
        let data_ref = DataReference::new(content).with_mapping_id("line-42");

        assert_eq!(data_ref.mapping_id(), Some("line-42"));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialization() {
        let content = Content::text("Test content");
        let data_ref = DataReference::new(content).with_mapping_id("test-mapping");

        let json = serde_json::to_string(&data_ref).unwrap();
        let deserialized: DataReference = serde_json::from_str(&json).unwrap();

        assert_eq!(data_ref.source_id(), deserialized.source_id());
        assert_eq!(data_ref.mapping_id(), deserialized.mapping_id());
    }
}
