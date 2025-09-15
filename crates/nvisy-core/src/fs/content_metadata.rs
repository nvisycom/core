//! Content metadata for filesystem operations
//!
//! This module provides the [`ContentMetadata`] struct for handling metadata
//! about content files, including paths, content types, and source tracking.

use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ContentKind, SupportedFormat};
use crate::path::ContentSource;

/// Metadata associated with content files
///
/// This struct stores metadata about content including its source identifier,
/// file path, and detected content kind based on file extension.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ContentMetadata {
    /// Unique identifier for the content source
    pub content_source: ContentSource,
    /// Optional path to the source file
    pub source_path: Option<PathBuf>,
}

impl ContentMetadata {
    /// Create new content metadata with just a source
    ///
    /// # Example
    ///
    /// ```
    /// use nvisy_core::{fs::ContentMetadata, ContentSource};
    ///
    /// let source = ContentSource::new();
    /// let metadata = ContentMetadata::new(source);
    /// ```
    pub fn new(content_source: ContentSource) -> Self {
        Self {
            content_source,
            source_path: None,
        }
    }

    /// Create content metadata with a file path
    ///
    /// # Example
    ///
    /// ```
    /// use nvisy_core::{fs::ContentMetadata, ContentSource};
    /// use std::path::PathBuf;
    ///
    /// let source = ContentSource::new();
    /// let metadata = ContentMetadata::with_path(source, PathBuf::from("document.pdf"));
    /// assert_eq!(metadata.file_extension(), Some("pdf"));
    /// ```
    pub fn with_path(content_source: ContentSource, path: impl Into<PathBuf>) -> Self {
        Self {
            content_source,
            source_path: Some(path.into()),
        }
    }

    /// Get the file extension if available
    pub fn file_extension(&self) -> Option<&str> {
        self.source_path
            .as_ref()
            .and_then(|path| path.extension())
            .and_then(|ext| ext.to_str())
    }

    /// Detect content kind from file extension
    ///
    /// # Example
    ///
    /// ```
    /// use nvisy_core::{fs::{ContentMetadata, ContentKind}, ContentSource};
    /// use std::path::PathBuf;
    ///
    /// let source = ContentSource::new();
    /// let metadata = ContentMetadata::with_path(source, PathBuf::from("image.png"));
    /// assert_eq!(metadata.content_kind(), Some(ContentKind::Image));
    /// ```
    pub fn content_kind(&self) -> ContentKind {
        self.file_extension()
            .map(ContentKind::from_file_extension)
            .unwrap_or_default()
    }

    /// Get the filename if available
    pub fn filename(&self) -> Option<&str> {
        self.source_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
    }

    /// Get the parent directory if available
    pub fn parent_directory(&self) -> Option<&Path> {
        self.source_path.as_ref().and_then(|path| path.parent())
    }

    /// Get the full path if available
    pub fn path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    /// Set the source path
    pub fn set_path(&mut self, path: impl Into<PathBuf>) {
        self.source_path = Some(path.into());
    }

    /// Remove the source path
    pub fn clear_path(&mut self) {
        self.source_path = None;
    }

    /// Check if this metadata has a path
    pub fn has_path(&self) -> bool {
        self.source_path.is_some()
    }

    /// Get the supported format if detectable from extension
    pub fn supported_format(&self) -> Option<SupportedFormat> {
        self.file_extension()
            .and_then(SupportedFormat::from_extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_metadata_creation() {
        let source = ContentSource::new();
        let metadata = ContentMetadata::new(source);

        assert_eq!(metadata.content_source, source);
        assert!(metadata.source_path.is_none());
        assert!(!metadata.has_path());
    }

    #[test]
    fn test_content_metadata_with_path() {
        let source = ContentSource::new();
        let path = PathBuf::from("/path/to/document.pdf");
        let metadata = ContentMetadata::with_path(source, path.clone());

        assert_eq!(metadata.content_source, source);
        assert_eq!(metadata.source_path, Some(path));
        assert!(metadata.has_path());
    }

    #[test]
    fn test_file_extension_detection() {
        let source = ContentSource::new();
        let metadata = ContentMetadata::with_path(source, PathBuf::from("document.pdf"));

        assert_eq!(metadata.file_extension(), Some("pdf"));
        assert_eq!(metadata.content_kind(), ContentKind::Document);
    }

    #[test]
    fn test_metadata_filename() {
        let source = ContentSource::new();
        let metadata = ContentMetadata::with_path(source, PathBuf::from("/path/to/file.txt"));

        assert_eq!(metadata.filename(), Some("file.txt"));
    }

    #[test]
    fn test_metadata_parent_directory() {
        let source = ContentSource::new();
        let metadata = ContentMetadata::with_path(source, PathBuf::from("/path/to/file.txt"));

        assert_eq!(metadata.parent_directory(), Some(Path::new("/path/to")));
    }

    #[test]
    fn test_path_operations() {
        let source = ContentSource::new();
        let mut metadata = ContentMetadata::new(source);

        assert!(!metadata.has_path());

        metadata.set_path("test.txt");
        assert!(metadata.has_path());
        assert_eq!(metadata.filename(), Some("test.txt"));

        metadata.clear_path();
        assert!(!metadata.has_path());
        assert_eq!(metadata.filename(), None);
    }

    #[test]
    fn test_supported_format_detection() {
        let source = ContentSource::new();
        let metadata = ContentMetadata::with_path(source, PathBuf::from("image.png"));

        assert_eq!(metadata.supported_format(), Some(SupportedFormat::Png));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_serialization() {
        let source = ContentSource::new();
        let metadata = ContentMetadata::with_path(source, PathBuf::from("test.json"));

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: ContentMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metadata, deserialized);
    }
}
