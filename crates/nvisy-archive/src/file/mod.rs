//! Archive file handling for content processing
//!
//! This module provides functionality for working with archive files,
//! including extraction to temporary directories and repacking from various sources.

pub mod archive_type;

use std::ffi::OsStr;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub use archive_type::ArchiveType;
use tempfile::TempDir;
use tokio::fs;

use crate::handler::ArchiveHandler;
use crate::{Error, Result};

/// Represents an archive file that can be loaded from various sources
///
/// This struct encapsulates an archive and provides methods for
/// extracting its contents to a temporary directory for processing.
#[derive(Debug)]
pub struct ArchiveFile {
    /// Type of archive
    pub archive_type: ArchiveType,
    /// Source data for the archive
    source: ArchiveSource,
}

/// Internal representation of archive data sources
#[derive(Debug)]
enum ArchiveSource {
    /// Archive loaded from a file path
    Path(PathBuf),
    /// Archive loaded from memory
    Memory(Vec<u8>),
    /// Archive loaded from an iterator
    Iterator(Vec<u8>),
}

impl ArchiveFile {
    /// Create a new archive file from a file path
    ///
    /// The archive type is automatically detected from the file extension.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nvisy_archive::ArchiveFile;
    /// use std::path::PathBuf;
    ///
    /// let archive = ArchiveFile::from_path("archive.zip")?;
    /// # Ok::<(), nvisy_archive::Error>(())
    /// ```
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .ok_or_else(|| Error::invalid_archive("No file extension found"))?;

        // Handle compound extensions like .tar.gz
        let full_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        let archive_type = if full_name.contains(".tar.") {
            // Try to match compound extensions first
            if let Some(pos) = full_name.find(".tar.") {
                let compound_ext = &full_name[pos + 1..]; // Skip the dot
                ArchiveType::from_file_extension(OsStr::new(compound_ext))
            } else {
                None
            }
        } else {
            None
        }
        .or_else(|| ArchiveType::from_file_extension(extension))
        .ok_or_else(|| Error::unsupported_format(extension.to_string_lossy().to_string()))?;

        Ok(Self {
            archive_type,
            source: ArchiveSource::Path(path.to_path_buf()),
        })
    }

    /// Create a new archive file from memory with explicit archive type
    ///
    /// # Example
    ///
    /// ```
    /// use nvisy_archive::{ArchiveFile, ArchiveType};
    ///
    /// let data = vec![0x50, 0x4B, 0x03, 0x04]; // ZIP signature
    /// let archive = ArchiveFile::from_memory(ArchiveType::Zip, data);
    /// ```
    pub fn from_memory(archive_type: ArchiveType, data: Vec<u8>) -> Self {
        Self {
            archive_type,
            source: ArchiveSource::Memory(data),
        }
    }

    /// Create a new archive file from an iterator of bytes
    ///
    /// The iterator will be consumed immediately and stored in memory.
    ///
    /// # Example
    ///
    /// ```
    /// use nvisy_archive::{ArchiveFile, ArchiveType};
    ///
    /// let data = [0x50, 0x4B, 0x03, 0x04]; // ZIP signature
    /// let archive = ArchiveFile::from_iterator(ArchiveType::Zip, data.into_iter());
    /// ```
    pub fn from_iterator(archive_type: ArchiveType, data: impl Iterator<Item = u8>) -> Self {
        let data: Vec<u8> = data.collect();
        Self {
            archive_type,
            source: ArchiveSource::Iterator(data),
        }
    }

    /// Create an archive with explicit type (useful for ambiguous extensions)
    pub fn with_archive_type(mut self, archive_type: ArchiveType) -> Self {
        self.archive_type = archive_type;
        self
    }

    /// Get the archive type
    pub fn archive_type(&self) -> ArchiveType {
        self.archive_type
    }

    /// Check if the archive source exists (only meaningful for file-based sources)
    pub async fn exists(&self) -> bool {
        match &self.source {
            ArchiveSource::Path(path) => fs::try_exists(path).await.unwrap_or(false),
            ArchiveSource::Memory(_) | ArchiveSource::Iterator(_) => true,
        }
    }

    /// Get the file path (if loaded from a file)
    pub fn path(&self) -> Option<&Path> {
        match &self.source {
            ArchiveSource::Path(path) => Some(path),
            _ => None,
        }
    }

    /// Get the size of the archive data
    pub async fn size(&self) -> Result<u64> {
        match &self.source {
            ArchiveSource::Path(path) => {
                let metadata = fs::metadata(path).await?;
                Ok(metadata.len())
            }
            ArchiveSource::Memory(data) | ArchiveSource::Iterator(data) => Ok(data.len() as u64),
        }
    }

    /// Extract the archive to a temporary directory
    ///
    /// This method extracts all contents of the archive to a temporary
    /// directory and returns an `ArchiveFileHandler` for managing the
    /// extracted contents.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The archive file cannot be read
    /// - The archive format is not supported
    /// - Extraction fails
    /// - Temporary directory creation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nvisy_archive::ArchiveFile;
    ///
    /// # async fn example() -> nvisy_archive::Result<()> {
    /// let archive = ArchiveFile::from_path("archive.zip")?;
    /// let handler = archive.unpack().await?;
    ///
    /// // Work with extracted files
    /// for file_path in handler.file_paths() {
    ///     println!("Found file: {:?}", file_path);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unpack(self) -> Result<ArchiveHandler> {
        // Create temporary directory
        let temp_dir = TempDir::new()
            .map_err(|e| Error::other(format!("Failed to create temporary directory: {}", e)))?;

        // Get archive data as bytes
        let data = self.get_data().await?;
        let cursor = Cursor::new(data);

        // Extract based on archive type
        let files = self.extract_archive(cursor, temp_dir.path()).await?;

        Ok(ArchiveHandler::new(
            self.archive_type,
            self.path().map(|p| p.to_path_buf()),
            temp_dir,
            files,
        ))
    }

    /// Get the archive data as bytes
    async fn get_data(&self) -> Result<Vec<u8>> {
        match &self.source {
            ArchiveSource::Path(path) => fs::read(path).await.map_err(Into::into),
            ArchiveSource::Memory(data) | ArchiveSource::Iterator(data) => Ok(data.clone()),
        }
    }

    /// Extract archive contents to the specified directory
    async fn extract_archive(
        &self,
        data: Cursor<Vec<u8>>,
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        match self.archive_type {
            ArchiveType::Zip => self.extract_zip(data, target_dir).await,
            ArchiveType::Tar => self.extract_tar(data, target_dir).await,
            ArchiveType::TarGz => self.extract_tar_gz(data, target_dir).await,
            ArchiveType::TarBz2 => self.extract_tar_bz2(data, target_dir).await,
            ArchiveType::TarXz => self.extract_tar_xz(data, target_dir).await,
            ArchiveType::Gz => self.extract_gz(data, target_dir).await,
            ArchiveType::Bz2 => self.extract_bz2(data, target_dir).await,
            ArchiveType::Xz => self.extract_xz(data, target_dir).await,
        }
    }

    /// Extract ZIP archive
    #[cfg(feature = "zip")]
    async fn extract_zip(&self, data: Cursor<Vec<u8>>, target_dir: &Path) -> Result<Vec<PathBuf>> {
        use tokio::io::AsyncWriteExt;
        use zip::ZipArchive;

        let mut archive = ZipArchive::new(data)?;
        let mut files = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = target_dir.join(file.name());

            // Create parent directories if they don't exist
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            if file.is_dir() {
                fs::create_dir_all(&file_path).await?;
            } else {
                let mut content = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut content)
                    .map_err(|e| Error::other(format!("Failed to read file from ZIP: {}", e)))?;

                let mut output_file = fs::File::create(&file_path).await?;
                output_file.write_all(&content).await?;
                files.push(file_path);
            }
        }

        Ok(files)
    }

    #[cfg(not(feature = "zip"))]
    async fn extract_zip(
        &self,
        _data: Cursor<Vec<u8>>,
        _target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        Err(Error::unsupported_format("ZIP support not enabled"))
    }

    /// Extract TAR archive
    #[cfg(feature = "tar")]
    async fn extract_tar(&self, data: Cursor<Vec<u8>>, target_dir: &Path) -> Result<Vec<PathBuf>> {
        use tar::Archive;
        use tokio::io::AsyncWriteExt;

        let mut archive = Archive::new(data);
        let mut files = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let file_path = target_dir.join(&path);

            // Create parent directories if they don't exist
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            if entry.header().entry_type().is_dir() {
                fs::create_dir_all(&file_path).await?;
            } else {
                let mut content = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut content)
                    .map_err(|e| Error::other(format!("Failed to read file from TAR: {}", e)))?;

                let mut output_file = fs::File::create(&file_path).await?;
                output_file.write_all(&content).await?;
                files.push(file_path);
            }
        }

        Ok(files)
    }

    #[cfg(not(feature = "tar"))]
    async fn extract_tar(
        &self,
        _data: Cursor<Vec<u8>>,
        _target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        Err(Error::unsupported_format("TAR support not enabled"))
    }

    /// Extract GZIP-compressed TAR archive
    async fn extract_tar_gz(
        &self,
        data: Cursor<Vec<u8>>,
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        use flate2::read::GzDecoder;
        let decoder = GzDecoder::new(data);
        let cursor = Cursor::new({
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut { decoder }, &mut buf)
                .map_err(|e| Error::other(format!("Failed to decompress GZIP: {}", e)))?;
            buf
        });
        self.extract_tar(cursor, target_dir).await
    }

    /// Extract BZIP2-compressed TAR archive
    async fn extract_tar_bz2(
        &self,
        data: Cursor<Vec<u8>>,
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        use bzip2::read::BzDecoder;
        let decoder = BzDecoder::new(data);
        let cursor = Cursor::new({
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut { decoder }, &mut buf)
                .map_err(|e| Error::other(format!("Failed to decompress BZIP2: {}", e)))?;
            buf
        });
        self.extract_tar(cursor, target_dir).await
    }

    /// Extract XZ-compressed TAR archive
    async fn extract_tar_xz(
        &self,
        data: Cursor<Vec<u8>>,
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        use xz2::read::XzDecoder;
        let mut decoder = XzDecoder::new(data);
        let mut decompressed_data = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut decompressed_data)
            .map_err(|e| Error::other(format!("Failed to decompress XZ: {}", e)))?;
        let cursor = Cursor::new(decompressed_data);
        self.extract_tar(cursor, target_dir).await
    }

    /// Extract single GZIP file
    async fn extract_gz(&self, data: Cursor<Vec<u8>>, target_dir: &Path) -> Result<Vec<PathBuf>> {
        use flate2::read::GzDecoder;
        use tokio::io::AsyncWriteExt;

        let mut decoder = GzDecoder::new(data);
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut content)
            .map_err(|e| Error::other(format!("Failed to decompress GZIP: {}", e)))?;

        // For single files, we need to determine the output filename
        let output_path = if let Some(path) = self.path() {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("extracted");
            target_dir.join(stem)
        } else {
            target_dir.join("extracted")
        };

        let mut output_file = fs::File::create(&output_path).await?;
        output_file.write_all(&content).await?;

        Ok(vec![output_path])
    }

    /// Extract single BZIP2 file
    async fn extract_bz2(&self, data: Cursor<Vec<u8>>, target_dir: &Path) -> Result<Vec<PathBuf>> {
        use bzip2::read::BzDecoder;
        use tokio::io::AsyncWriteExt;

        let mut decoder = BzDecoder::new(data);
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut content)
            .map_err(|e| Error::other(format!("Failed to decompress BZIP2: {}", e)))?;

        let output_path = if let Some(path) = self.path() {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("extracted");
            target_dir.join(stem)
        } else {
            target_dir.join("extracted")
        };

        let mut output_file = fs::File::create(&output_path).await?;
        output_file.write_all(&content).await?;

        Ok(vec![output_path])
    }

    /// Extract single XZ file
    async fn extract_xz(&self, data: Cursor<Vec<u8>>, target_dir: &Path) -> Result<Vec<PathBuf>> {
        use tokio::io::AsyncWriteExt;
        use xz2::read::XzDecoder;

        let mut decoder = XzDecoder::new(data);
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut content)
            .map_err(|e| Error::other(format!("Failed to decompress XZ: {}", e)))?;

        let output_path = if let Some(path) = self.path() {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("extracted");
            target_dir.join(stem)
        } else {
            target_dir.join("extracted")
        };

        let mut output_file = fs::File::create(&output_path).await?;
        output_file.write_all(&content).await?;

        Ok(vec![output_path])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_file_from_memory() {
        let data = vec![0x50, 0x4B, 0x03, 0x04]; // ZIP signature
        let archive = ArchiveFile::from_memory(ArchiveType::Zip, data);
        assert_eq!(archive.archive_type(), ArchiveType::Zip);
        assert!(archive.path().is_none());
    }

    #[test]
    fn test_archive_file_from_iterator() {
        let data = [0x50, 0x4B, 0x03, 0x04]; // ZIP signature
        let archive = ArchiveFile::from_iterator(ArchiveType::Zip, data.into_iter());
        assert_eq!(archive.archive_type(), ArchiveType::Zip);
    }

    #[test]
    fn test_archive_file_from_path() -> Result<()> {
        let archive = ArchiveFile::from_path("test.zip")?;
        assert_eq!(archive.archive_type(), ArchiveType::Zip);
        assert!(archive.path().is_some());
        Ok(())
    }

    #[test]
    fn test_compound_extension() -> Result<()> {
        let archive = ArchiveFile::from_path("test.tar.gz")?;
        assert_eq!(archive.archive_type(), ArchiveType::TarGz);
        Ok(())
    }

    #[test]
    fn test_unsupported_extension() {
        let result = ArchiveFile::from_path("test.unknown");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_memory_size() {
        let data = vec![1, 2, 3, 4, 5];
        let archive = ArchiveFile::from_memory(ArchiveType::Zip, data);
        assert_eq!(archive.size().await.unwrap(), 5);
    }
}
