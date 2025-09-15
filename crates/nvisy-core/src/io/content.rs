//! Content types supported by the Nvisy system
//!
//! This module provides the Content enum for representing different types
//! of data content within the system.

use bytes::Bytes;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Content types supported by the Nvisy system
///
/// Simplified content representation for efficient processing.
///
/// # Examples
///
/// ```rust
/// use nvisy_core::Content;
/// use bytes::Bytes;
///
/// let text_content = Content::Text("Sample text".to_string());
/// let binary_content = Content::Binary {
///     data: Bytes::from(vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]),
///     mime_type: "application/octet-stream".to_string(),
/// };
///
/// assert!(text_content.is_textual());
/// assert!(!binary_content.is_textual());
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Content {
    /// Text content stored as UTF-8 string
    Text(String),

    /// Generic binary content with MIME type
    Binary {
        /// Raw binary data
        data: Bytes,
        /// MIME type describing the content
        mime_type: String,
    },

    /// Empty or null content
    Empty,
}

impl Content {
    /// Get the type name of this content
    pub fn type_name(&self) -> &'static str {
        match self {
            Content::Text(_) => "text",
            Content::Binary { .. } => "binary",
            Content::Empty => "empty",
        }
    }

    /// Check if this content is textual
    pub fn is_textual(&self) -> bool {
        matches!(self, Content::Text(_))
    }

    /// Check if this content is multimedia (audio, video, image)
    pub fn is_multimedia(&self) -> bool {
        false // Simplified - no specific multimedia types
    }

    /// Check if this content has binary data
    pub fn has_binary_data(&self) -> bool {
        !matches!(self, Content::Text(_) | Content::Empty)
    }

    /// Get the estimated size in bytes
    pub fn estimated_size(&self) -> usize {
        match self {
            Content::Text(text) => text.len(),
            Content::Binary { data, .. } => data.len(),
            Content::Empty => 0,
        }
    }

    /// Get the format/MIME type of this content
    pub fn format(&self) -> Option<&str> {
        match self {
            Content::Text(_) => Some("text/plain"),
            Content::Binary { mime_type, .. } => Some(mime_type),
            Content::Empty => None,
        }
    }

    /// Extract raw bytes from content, if available
    pub fn as_bytes(&self) -> Option<&Bytes> {
        match self {
            Content::Binary { data, .. } => Some(data),
            Content::Text(_) | Content::Empty => None,
        }
    }

    /// Extract text from content, if it's textual
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Create text content
    pub fn text<S: Into<String>>(content: S) -> Self {
        Content::Text(content.into())
    }

    /// Create binary content
    pub fn binary<S: Into<String>>(data: Bytes, mime_type: S) -> Self {
        Content::Binary {
            data,
            mime_type: mime_type.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_types() {
        let text = Content::text("Hello");
        assert!(text.is_textual());
        assert!(!text.is_multimedia());
        assert!(!text.has_binary_data());
        assert_eq!(text.type_name(), "text");
        assert_eq!(text.format(), Some("text/plain"));

        let binary_data = Bytes::from(vec![1, 2, 3, 4]);
        let binary = Content::binary(binary_data, "application/octet-stream");
        assert!(!binary.is_textual());
        assert!(!binary.is_multimedia());
        assert!(binary.has_binary_data());
        assert_eq!(binary.type_name(), "binary");
    }

    #[test]
    fn test_content_size_estimation() {
        let text = Content::text("Hello, world!");
        assert_eq!(text.estimated_size(), 13);

        let binary_data = Bytes::from(vec![0; 100]);
        let binary = Content::binary(binary_data, "application/octet-stream");
        assert_eq!(binary.estimated_size(), 100);

        let empty = Content::Empty;
        assert_eq!(empty.estimated_size(), 0);
    }

    #[test]
    fn test_content_data_access() {
        let text_content = Content::text("Hello");
        assert_eq!(text_content.as_text(), Some("Hello"));
        assert!(text_content.as_bytes().is_none());

        let binary_data = Bytes::from(vec![1, 2, 3]);
        let binary_content = Content::binary(binary_data.clone(), "test");
        assert_eq!(binary_content.as_bytes(), Some(&binary_data));
        assert!(binary_content.as_text().is_none());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialization() {
        let content = Content::text("Test content");

        let json = serde_json::to_string(&content).unwrap();
        let deserialized: Content = serde_json::from_str(&json).unwrap();

        assert_eq!(content, deserialized);
    }
}
