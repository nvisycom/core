//! Archive type definitions and utilities
//!
//! This module defines the different archive formats supported by the library
//! and provides utilities for working with archive types.

use std::ffi::OsStr;
use std::fmt;

/// Supported archive types
///
/// This enum represents the different archive formats that can be processed.
/// It provides methods to determine the archive type from file extensions
/// and to get the supported extensions for each type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchiveType {
    /// ZIP archive format
    Zip,
    /// TAR archive format (uncompressed)
    Tar,
    /// GZIP compressed TAR archive
    TarGz,
    /// BZIP2 compressed TAR archive
    TarBz2,
    /// XZ compressed TAR archive
    TarXz,
    /// GZIP compression (single file)
    Gz,
    /// BZIP2 compression (single file)
    Bz2,
    /// XZ compression (single file)
    Xz,
}

impl ArchiveType {
    /// Determine archive type from file extension
    ///
    /// # Arguments
    ///
    /// * `extension` - File extension string (without the dot)
    ///
    /// # Returns
    ///
    /// `Some(ArchiveType)` if the extension is recognized, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use nvisy_archive::ArchiveType;
    ///
    /// assert_eq!(ArchiveType::from_file_extension("zip"), Some(ArchiveType::Zip));
    /// assert_eq!(ArchiveType::from_file_extension("tar.gz"), Some(ArchiveType::TarGz));
    /// assert_eq!(ArchiveType::from_file_extension("unknown"), None);
    /// ```
    pub fn from_file_extension(extension: &OsStr) -> Option<Self> {
        let extension_str = extension.to_str()?.to_lowercase();
        match extension_str.as_str() {
            "zip" => Some(Self::Zip),
            "tar" => Some(Self::Tar),
            "tar.gz" | "tgz" => Some(Self::TarGz),
            "tar.bz2" | "tbz2" | "tb2" => Some(Self::TarBz2),
            "tar.xz" | "txz" => Some(Self::TarXz),
            "gz" | "gzip" => Some(Self::Gz),
            "bz2" | "bzip2" => Some(Self::Bz2),
            "xz" => Some(Self::Xz),
            _ => None,
        }
    }

    /// Get the file extensions associated with this archive type
    ///
    /// Returns a slice of static string references representing all
    /// the file extensions that correspond to this archive type.
    ///
    /// # Examples
    ///
    /// ```
    /// use nvisy_archive::ArchiveType;
    ///
    /// assert_eq!(ArchiveType::Zip.file_extensions(), &["zip"]);
    /// assert_eq!(ArchiveType::TarGz.file_extensions(), &["tar.gz", "tgz"]);
    /// ```
    pub fn file_extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Zip => &["zip"],
            Self::Tar => &["tar"],
            Self::TarGz => &["tar.gz", "tgz"],
            Self::TarBz2 => &["tar.bz2", "tbz2", "tb2"],
            Self::TarXz => &["tar.xz", "txz"],
            Self::Gz => &["gz", "gzip"],
            Self::Bz2 => &["bz2", "bzip2"],
            Self::Xz => &["xz"],
        }
    }

    /// Get the primary file extension for this archive type
    ///
    /// Returns the most common/preferred file extension for this archive type.
    ///
    /// # Examples
    ///
    /// ```
    /// use nvisy_archive::ArchiveType;
    ///
    /// assert_eq!(ArchiveType::Zip.primary_extension(), "zip");
    /// assert_eq!(ArchiveType::TarGz.primary_extension(), "tar.gz");
    /// ```
    pub fn primary_extension(&self) -> &'static str {
        self.file_extensions()[0]
    }

    /// Check if this archive type is a compressed TAR variant
    pub fn is_tar_variant(&self) -> bool {
        matches!(self, Self::Tar | Self::TarGz | Self::TarBz2 | Self::TarXz)
    }

    /// Check if this archive type supports multiple files
    pub fn supports_multiple_files(&self) -> bool {
        matches!(
            self,
            Self::Zip | Self::Tar | Self::TarGz | Self::TarBz2 | Self::TarXz
        )
    }
}

impl fmt::Display for ArchiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip => write!(f, "ZIP"),
            Self::Tar => write!(f, "TAR"),
            Self::TarGz => write!(f, "TAR.GZ"),
            Self::TarBz2 => write!(f, "TAR.BZ2"),
            Self::TarXz => write!(f, "TAR.XZ"),
            Self::Gz => write!(f, "GZIP"),
            Self::Bz2 => write!(f, "BZIP2"),
            Self::Xz => write!(f, "XZ"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_type_from_extension() {
        assert_eq!(
            ArchiveType::from_file_extension(OsStr::new("zip")),
            Some(ArchiveType::Zip)
        );
        assert_eq!(
            ArchiveType::from_file_extension(OsStr::new("ZIP")),
            Some(ArchiveType::Zip)
        );
        assert_eq!(
            ArchiveType::from_file_extension(OsStr::new("tar")),
            Some(ArchiveType::Tar)
        );
        assert_eq!(
            ArchiveType::from_file_extension(OsStr::new("tar.gz")),
            Some(ArchiveType::TarGz)
        );
        assert_eq!(
            ArchiveType::from_file_extension(OsStr::new("tgz")),
            Some(ArchiveType::TarGz)
        );
        assert_eq!(
            ArchiveType::from_file_extension(OsStr::new("unknown")),
            None
        );
    }

    #[test]
    fn test_archive_type_extensions() {
        assert_eq!(ArchiveType::Zip.file_extensions(), &["zip"]);
        assert_eq!(ArchiveType::TarGz.file_extensions(), &["tar.gz", "tgz"]);
        assert!(ArchiveType::TarBz2.file_extensions().contains(&"tar.bz2"));
    }

    #[test]
    fn test_archive_type_primary_extension() {
        assert_eq!(ArchiveType::Zip.primary_extension(), "zip");
        assert_eq!(ArchiveType::TarGz.primary_extension(), "tar.gz");
    }

    #[test]
    fn test_archive_type_variants() {
        assert!(ArchiveType::Tar.is_tar_variant());
        assert!(ArchiveType::TarGz.is_tar_variant());
        assert!(!ArchiveType::Zip.is_tar_variant());
        assert!(!ArchiveType::Gz.is_tar_variant());
    }

    #[test]
    fn test_archive_type_multiple_files() {
        assert!(ArchiveType::Zip.supports_multiple_files());
        assert!(ArchiveType::Tar.supports_multiple_files());
        assert!(!ArchiveType::Gz.supports_multiple_files());
        assert!(!ArchiveType::Bz2.supports_multiple_files());
    }

    #[test]
    fn test_archive_type_display() {
        assert_eq!(ArchiveType::Zip.to_string(), "ZIP");
        assert_eq!(ArchiveType::TarGz.to_string(), "TAR.GZ");
    }
}
