//! Filesystem module for content file operations
//!
//! This module provides filesystem-specific functionality for working with
//! content files, including file metadata handling and archive operations.
//!
//! # Core Types
//!
//! - [`ContentFile`]: A file wrapper that combines filesystem operations with content tracking
//! - [`ContentFileMetadata`]: Metadata information for content files

//!
//! # Example
//!
//! ```no_run
//! use nvisy_core::fs::ContentFile;
//! use nvisy_core::io::ContentData;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new file
//!     let mut content_file = ContentFile::create("example.txt").await?;
//!
//!     // Write some content
//!     let content_data = ContentData::from("Hello, world!");
//!     let metadata = content_file.write_from_content_data(content_data).await?;
//!
//!     println!("Written to: {:?}", metadata.source_path);
//!     Ok(())
//! }
//! ```

mod content_file;
mod content_kind;
mod content_metadata;
mod data_sensitivity;
mod data_structure_kind;
mod supported_format;

use std::path::PathBuf;

// Re-export main types
pub use content_file::ContentFile;
pub use content_kind::ContentKind;
pub use content_metadata::ContentMetadata;
pub use data_sensitivity::DataSensitivity;
pub use data_structure_kind::DataStructureKind;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
pub use supported_format::SupportedFormat;

use crate::path::ContentSource;

/// Metadata information for content files
///
/// TODO: Implement comprehensive file metadata handling including:
/// - File timestamps (created, modified, accessed)
/// - File permissions and ownership
/// - File size and disk usage
/// - Extended attributes
/// - Content type detection beyond extensions
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ContentFileMetadata {
    /// Content source identifier
    pub content_source: ContentSource,
    /// Path to the file
    pub path: PathBuf,
    /// Detected content kind
    pub content_kind: Option<ContentKind>,
    /// File size in bytes
    pub size: Option<u64>,
    // TODO: Add more metadata fields
}

impl ContentFileMetadata {
    /// Create new file metadata
    pub fn new(content_source: ContentSource, path: PathBuf) -> Self {
        Self {
            content_source,
            path,
            content_kind: None,
            size: None,
        }
    }

    /// Set the content kind
    pub fn with_content_kind(mut self, kind: ContentKind) -> Self {
        self.content_kind = Some(kind);
        self
    }

    /// Set the file size
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_file_metadata() {
        let source = ContentSource::new();
        let path = PathBuf::from("test.txt");

        let metadata = ContentFileMetadata::new(source, path.clone())
            .with_content_kind(ContentKind::Text)
            .with_size(1024);

        assert_eq!(metadata.content_source, source);
        assert_eq!(metadata.path, path);
        assert_eq!(metadata.content_kind, Some(ContentKind::Text));
        assert_eq!(metadata.size, Some(1024));
    }
}
