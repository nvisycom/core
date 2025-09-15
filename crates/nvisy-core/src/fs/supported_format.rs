//! Supported file format definitions and utilities
//!
//! This module provides the [`SupportedFormat`] struct and related enums
//! for identifying and categorizing different file formats supported by nvisy.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use crate::fs::{ContentKind, DataStructureKind};

/// Individual supported formats with their categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, EnumIter)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[strum(serialize_all = "lowercase")]
pub enum SupportedFormat {
    // Text formats
    /// Plain text files (.txt)
    Txt,
    /// XML documents (.xml)
    Xml,
    /// JSON data files (.json)
    Json,
    /// Comma-separated values (.csv)
    Csv,

    // Document formats
    /// PDF documents (.pdf)
    Pdf,
    /// Microsoft Word legacy format (.doc)
    Doc,
    /// Microsoft Word modern format (.docx)
    Docx,
    /// Rich Text Format (.rtf)
    Rtf,

    // Image formats
    /// JPEG images (.jpg)
    Jpg,
    /// JPEG images (.jpeg)
    Jpeg,
    /// PNG images (.png)
    Png,
    /// SVG vector graphics (.svg)
    Svg,
}

impl SupportedFormat {
    /// Get the content kind category for this format
    pub const fn content_kind(self) -> ContentKind {
        match self {
            Self::Txt | Self::Xml | Self::Json | Self::Csv => ContentKind::Text,
            Self::Pdf | Self::Doc | Self::Docx | Self::Rtf => ContentKind::Document,
            Self::Jpg | Self::Jpeg | Self::Png | Self::Svg => ContentKind::Image,
        }
    }

    /// Get the primary file extension for this format
    pub const fn primary_extension(self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Xml => "xml",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Pdf => "pdf",
            Self::Doc => "doc",
            Self::Docx => "docx",
            Self::Rtf => "rtf",
            Self::Jpg => "jpg",
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::Svg => "svg",
        }
    }

    /// Get all possible file extensions for this format
    pub const fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Txt => &["txt", "text"],
            Self::Xml => &["xml"],
            Self::Json => &["json"],
            Self::Csv => &["csv"],
            Self::Pdf => &["pdf"],
            Self::Doc => &["doc"],
            Self::Docx => &["docx"],
            Self::Rtf => &["rtf"],
            Self::Jpg => &["jpg", "jpeg"],
            Self::Jpeg => &["jpeg", "jpg"],
            Self::Png => &["png"],
            Self::Svg => &["svg"],
        }
    }

    /// Attempt to identify a format from a file extension
    ///
    /// # Example
    ///
    /// ```
    /// use nvisy_core::fs::SupportedFormat;
    ///
    /// assert_eq!(SupportedFormat::from_extension("txt"), Some(SupportedFormat::Txt));
    /// assert_eq!(SupportedFormat::from_extension("jpeg"), Some(SupportedFormat::Jpeg));
    /// assert_eq!(SupportedFormat::from_extension("unknown"), None);
    /// ```
    pub fn from_extension(extension: &str) -> Option<Self> {
        let ext = extension.to_lowercase();
        match ext.as_str() {
            "txt" | "text" => Some(Self::Txt),
            "xml" => Some(Self::Xml),
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "pdf" => Some(Self::Pdf),
            "doc" => Some(Self::Doc),
            "docx" => Some(Self::Docx),
            "rtf" => Some(Self::Rtf),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "svg" => Some(Self::Svg),
            _ => None,
        }
    }

    /// Check if this format is text-based
    pub const fn is_text(self) -> bool {
        matches!(self.content_kind(), ContentKind::Text)
    }

    /// Check if this format is a document format
    pub const fn is_document(self) -> bool {
        matches!(self.content_kind(), ContentKind::Document)
    }

    /// Check if this format is an image format
    pub const fn is_image(self) -> bool {
        matches!(self.content_kind(), ContentKind::Image)
    }

    /// Get a human-readable description of the format
    pub const fn description(self) -> &'static str {
        match self {
            Self::Txt => "Plain text file",
            Self::Xml => "XML document",
            Self::Json => "JSON data",
            Self::Csv => "Comma-separated values",
            Self::Pdf => "PDF document",
            Self::Doc => "Microsoft Word document (legacy)",
            Self::Docx => "Microsoft Word document",
            Self::Rtf => "Rich Text Format",
            Self::Jpg => "JPEG image",
            Self::Jpeg => "JPEG image",
            Self::Png => "PNG image",
            Self::Svg => "SVG vector image",
        }
    }

    /// Get the MIME type for this format
    pub const fn mime_type(self) -> &'static str {
        match self {
            Self::Txt => "text/plain",
            Self::Xml => "application/xml",
            Self::Json => "application/json",
            Self::Csv => "text/csv",
            Self::Pdf => "application/pdf",
            Self::Doc => "application/msword",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Rtf => "application/rtf",
            Self::Jpg => "image/jpeg",
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Svg => "image/svg+xml",
        }
    }

    /// Get the data structure kind for this format
    pub const fn data_structure_kind(self) -> DataStructureKind {
        match self {
            // Highly structured formats with defined schemas
            Self::Xml | Self::Json => DataStructureKind::HighlyStructured,
            // Semi-structured formats with some organization
            Self::Csv => DataStructureKind::SemiStructured,
            // Unstructured formats
            Self::Txt
            | Self::Pdf
            | Self::Doc
            | Self::Docx
            | Self::Rtf
            | Self::Jpg
            | Self::Jpeg
            | Self::Png
            | Self::Svg => DataStructureKind::Unstructured,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_kind_classification() {
        assert_eq!(SupportedFormat::Txt.content_kind(), ContentKind::Text);
        assert_eq!(SupportedFormat::Json.content_kind(), ContentKind::Text);
        assert_eq!(SupportedFormat::Pdf.content_kind(), ContentKind::Document);
        assert_eq!(SupportedFormat::Png.content_kind(), ContentKind::Image);
    }

    #[test]
    fn test_extension_detection() {
        assert_eq!(
            SupportedFormat::from_extension("txt"),
            Some(SupportedFormat::Txt)
        );
        assert_eq!(
            SupportedFormat::from_extension("TXT"),
            Some(SupportedFormat::Txt)
        );
        assert_eq!(
            SupportedFormat::from_extension("jpeg"),
            Some(SupportedFormat::Jpeg)
        );
        assert_eq!(
            SupportedFormat::from_extension("jpg"),
            Some(SupportedFormat::Jpeg)
        );
        assert_eq!(SupportedFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_format_predicates() {
        assert!(SupportedFormat::Txt.is_text());
        assert!(!SupportedFormat::Txt.is_document());
        assert!(!SupportedFormat::Txt.is_image());

        assert!(!SupportedFormat::Pdf.is_text());
        assert!(SupportedFormat::Pdf.is_document());
        assert!(!SupportedFormat::Pdf.is_image());

        assert!(!SupportedFormat::Png.is_text());
        assert!(!SupportedFormat::Png.is_document());
        assert!(SupportedFormat::Png.is_image());
    }

    #[test]
    fn test_extensions() {
        assert!(SupportedFormat::Txt.extensions().contains(&"txt"));
        assert!(SupportedFormat::Jpeg.extensions().contains(&"jpg"));
        assert!(SupportedFormat::Jpeg.extensions().contains(&"jpeg"));
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(SupportedFormat::Txt.mime_type(), "text/plain");
        assert_eq!(SupportedFormat::Json.mime_type(), "application/json");
        assert_eq!(SupportedFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(SupportedFormat::Png.mime_type(), "image/png");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialization() {
        let format = SupportedFormat::Json;
        let serialized = serde_json::to_string(&format).unwrap();
        assert_eq!(serialized, "\"json\"");

        let deserialized: SupportedFormat = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, format);
    }
}
