//! Archive handling library for nvisy
//!
//! This crate provides functionality for working with various archive formats
//! including ZIP, TAR, and other compressed archive types. It supports both
//! reading from files and memory, with flexible loading options.

pub mod file;
pub mod handler;

// Re-exports for convenience
pub use file::{ArchiveFile, ArchiveType};
pub use handler::ArchiveHandler;

/// Archive processing errors
///
/// This enum represents all the possible errors that can occur during
/// archive operations, including I/O errors, format-specific errors,
/// and general processing errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O related errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP format errors
    #[cfg(feature = "zip")]
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Archive format not supported
    #[error("Unsupported archive format: {format}")]
    UnsupportedFormat { format: String },

    /// Invalid archive structure or data
    #[error("Invalid archive: {message}")]
    InvalidArchive { message: String },

    /// Entry not found in archive
    #[error("Entry not found: {name}")]
    EntryNotFound { name: String },

    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    /// Archive is corrupted or incomplete
    #[error("Corrupted archive: {message}")]
    Corrupted { message: String },

    /// Memory or resource limits exceeded
    #[error("Resource limit exceeded: {message}")]
    ResourceLimit { message: String },

    /// Generic error with custom message
    #[error("{message}")]
    Other { message: String },
}

impl Error {
    /// Create a new unsupported format error
    pub fn unsupported_format(format: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            format: format.into(),
        }
    }

    /// Create a new invalid archive error
    pub fn invalid_archive(message: impl Into<String>) -> Self {
        Self::InvalidArchive {
            message: message.into(),
        }
    }

    /// Create a new entry not found error
    pub fn entry_not_found(name: impl Into<String>) -> Self {
        Self::EntryNotFound { name: name.into() }
    }

    /// Create a new permission denied error
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            message: message.into(),
        }
    }

    /// Create a new corrupted archive error
    pub fn corrupted(message: impl Into<String>) -> Self {
        Self::Corrupted {
            message: message.into(),
        }
    }

    /// Create a new resource limit error
    pub fn resource_limit(message: impl Into<String>) -> Self {
        Self::ResourceLimit {
            message: message.into(),
        }
    }

    /// Create a new generic error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}

/// Result type alias for archive operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = Error::unsupported_format("custom");
        assert!(matches!(error, Error::UnsupportedFormat { .. }));

        let error = Error::invalid_archive("test message");
        assert!(matches!(error, Error::InvalidArchive { .. }));

        let error = Error::entry_not_found("missing.txt");
        assert!(matches!(error, Error::EntryNotFound { .. }));

        let error = Error::permission_denied("access denied");
        assert!(matches!(error, Error::PermissionDenied { .. }));

        let error = Error::corrupted("bad data");
        assert!(matches!(error, Error::Corrupted { .. }));

        let error = Error::resource_limit("too big");
        assert!(matches!(error, Error::ResourceLimit { .. }));

        let error = Error::other("generic error");
        assert!(matches!(error, Error::Other { .. }));
    }

    #[test]
    fn test_error_display() {
        let error = Error::unsupported_format("test");
        assert!(error.to_string().contains("Unsupported archive format"));

        let error = Error::invalid_archive("bad archive");
        assert!(error.to_string().contains("Invalid archive"));
    }
}
