//! Content type classification for different categories of data
//!
//! This module provides the [`ContentKind`] enum for classifying content
//! based on file extensions and supported formats.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

use super::SupportedFormat;

/// Content type classification for different categories of data
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Display, EnumString, EnumIter)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum ContentKind {
    /// Plain text content
    Text,
    /// Document files (PDF, Word, etc.)
    Document,
    /// Image files
    Image,
    /// Unknown or unsupported content type
    #[default]
    Unknown,
}

impl ContentKind {
    /// Detect content kind from file extension
    pub fn from_file_extension(extension: &str) -> Self {
        SupportedFormat::from_extension(extension)
            .map(|format| format.content_kind())
            .unwrap_or(ContentKind::Unknown)
    }

    /// Check if this content kind represents text-based content
    pub fn is_text_based(&self) -> bool {
        matches!(self, ContentKind::Text)
    }

    /// Get supported file extensions for this content kind
    pub fn file_extensions(&self) -> Vec<&'static str> {
        if matches!(self, ContentKind::Unknown) {
            return vec![];
        }

        SupportedFormat::iter()
            .filter(|format| format.content_kind() == *self)
            .flat_map(|format| format.extensions())
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_kind_from_extension() {
        assert_eq!(ContentKind::from_file_extension("txt"), ContentKind::Text);
        assert_eq!(ContentKind::from_file_extension("TXT"), ContentKind::Text);
        assert_eq!(
            ContentKind::from_file_extension("pdf"),
            ContentKind::Document
        );
        assert_eq!(ContentKind::from_file_extension("png"), ContentKind::Image);
        assert_eq!(
            ContentKind::from_file_extension("unknown"),
            ContentKind::Unknown
        );
    }

    #[test]
    fn test_content_kind_file_extensions() {
        let extensions = ContentKind::Image.file_extensions();
        assert!(extensions.contains(&"png"));
        assert!(extensions.contains(&"jpg"));

        let txt_extensions = ContentKind::Text.file_extensions();
        assert!(txt_extensions.contains(&"txt"));
    }

    #[test]
    fn test_content_kind_display() {
        assert_eq!(ContentKind::Text.to_string(), "text");
        assert_eq!(ContentKind::Document.to_string(), "document");
        assert_eq!(ContentKind::Image.to_string(), "image");
        assert_eq!(ContentKind::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_content_kind_text_classification() {
        assert!(ContentKind::Text.is_text_based());
        assert!(!ContentKind::Document.is_text_based());
        assert!(!ContentKind::Unknown.is_text_based());
        assert!(!ContentKind::Image.is_text_based());
    }

    #[test]
    fn test_case_insensitive_extension_detection() {
        assert_eq!(ContentKind::from_file_extension("TXT"), ContentKind::Text);
        assert_eq!(
            ContentKind::from_file_extension("PDF"),
            ContentKind::Document
        );
        assert_eq!(ContentKind::from_file_extension("PNG"), ContentKind::Image);
    }

    #[test]
    fn test_default() {
        assert_eq!(ContentKind::default(), ContentKind::Unknown);
    }
}
