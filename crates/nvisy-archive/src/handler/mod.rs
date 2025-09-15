//! Archive file handler for managing extracted archive contents
//!
//! This module provides the [`ArchiveFileHandler`] struct for managing
//! temporary directories containing extracted archive contents and
//! repacking them back into archives.

pub mod tar_handler;
pub mod zip_handler;

use std::fs;
use std::path::{Path, PathBuf};

// Re-exports for convenience
pub use tar_handler::{TarArchiveBuilder, TarArchiveHandler, TarEntryInfo};
use tempfile::TempDir;
pub use zip_handler::{ZipArchiveBuilder, ZipArchiveHandler, ZipEntryInfo};

use crate::{ArchiveType, Error, Result};

/// Handler for unpacked archive contents
///
/// This struct manages the temporary directory containing extracted
/// archive contents and provides methods for iterating over files
/// and repacking the archive.
#[derive(Debug)]
pub struct ArchiveHandler {
    /// Type of the original archive
    pub archive_type: ArchiveType,
    /// Original archive file path (if loaded from file)
    pub original_path: Option<PathBuf>,
    /// Temporary directory containing extracted files
    temp_dir: TempDir,
    /// Files found in the archive
    files: Vec<PathBuf>,
}

impl ArchiveHandler {
    /// Create a new archive file handler
    ///
    /// This is typically called internally by `ArchiveFile::unpack()`.
    pub fn new(
        archive_type: ArchiveType,
        original_path: Option<PathBuf>,
        temp_dir: TempDir,
        files: Vec<PathBuf>,
    ) -> Self {
        Self {
            archive_type,
            original_path,
            temp_dir,
            files,
        }
    }

    /// Get the path to the temporary directory containing extracted files
    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Get the number of files in the archive
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Check if the archive is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get a list of all file paths in the archive
    pub fn file_paths(&self) -> &[PathBuf] {
        &self.files
    }

    /// Find files matching a specific predicate
    pub fn find_files(&self, predicate: impl Fn(&PathBuf) -> bool) -> Vec<&PathBuf> {
        self.files.iter().filter(|path| predicate(path)).collect()
    }

    /// Find files with specific extension
    pub fn find_files_by_extension(&self, extension: &str) -> Vec<&PathBuf> {
        self.find_files(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case(extension))
                .unwrap_or(false)
        })
    }

    /// Get all files recursively in the temporary directory
    pub fn refresh_file_list(&mut self) -> Result<()> {
        self.files = Self::scan_files(self.temp_path())?;
        Ok(())
    }

    /// Create a new archive from the current temporary directory contents
    ///
    /// This method packages all files in the temporary directory back into
    /// an archive file at the specified location.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The target directory cannot be created
    /// - Archive creation fails
    /// - File I/O operations fail
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nvisy_archive::{ArchiveFile, ArchiveType};
    ///
    /// # async fn example() -> nvisy_archive::Result<()> {
    /// let archive = ArchiveFile::from_path("original.zip")?;
    /// let handler = archive.unpack().await?;
    ///
    /// // Modify files in handler.temp_path()...
    ///
    /// let new_archive = handler.pack("modified.zip").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pack(self, target_path: impl AsRef<Path>) -> Result<crate::ArchiveFile> {
        let target_path = target_path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| Error::other(format!("Failed to create parent directory: {}", e)))?;
        }

        // Determine archive type from target path extension or use original type
        let archive_type = target_path
            .extension()
            .and_then(ArchiveType::from_file_extension)
            .unwrap_or(self.archive_type);

        match archive_type {
            ArchiveType::Zip => {
                #[cfg(feature = "zip")]
                {
                    let zip_handler = zip_handler::ZipArchiveBuilder::for_directory();
                    zip_handler
                        .create_from_directory(self.temp_path(), target_path)
                        .await?;
                }
                #[cfg(not(feature = "zip"))]
                {
                    return Err(Error::unsupported_format("ZIP support not enabled"));
                }
            }
            ArchiveType::Tar | ArchiveType::TarGz | ArchiveType::TarBz2 | ArchiveType::TarXz => {
                #[cfg(feature = "tar")]
                {
                    let tar_handler = tar_handler::TarArchiveBuilder::for_directory(archive_type);
                    tar_handler
                        .create_from_directory(self.temp_path(), target_path)
                        .await?;
                }
                #[cfg(not(feature = "tar"))]
                {
                    return Err(Error::unsupported_format("TAR support not enabled"));
                }
            }
            _ => {
                return Err(Error::unsupported_format(format!(
                    "Packing format not supported: {}",
                    archive_type
                )));
            }
        }

        crate::ArchiveFile::from_path(target_path)
    }

    /// Scan the directory for files recursively
    pub fn scan_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let entries = fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                // Recursively scan subdirectories
                let mut sub_files = Self::scan_files(&path)?;
                files.append(&mut sub_files);
            }
        }

        files.sort();
        Ok(files)
    }

    /// Get relative paths of all files (relative to temp directory)
    pub fn relative_file_paths(&self) -> Result<Vec<PathBuf>> {
        let temp_path = self.temp_path();
        self.files
            .iter()
            .map(|path| {
                path.strip_prefix(temp_path)
                    .map(|p| p.to_path_buf())
                    .map_err(|e| Error::other(format!("Invalid file path: {}", e)))
            })
            .collect()
    }

    /// Check if a specific file exists in the archive
    pub fn contains_file(&self, relative_path: impl AsRef<Path>) -> bool {
        let target_path = self.temp_path().join(relative_path);
        self.files.contains(&target_path)
    }

    /// Get the content of a specific file as bytes
    pub async fn read_file(&self, relative_path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let target_path = self.temp_path().join(relative_path);
        if !self.files.contains(&target_path) {
            return Err(Error::entry_not_found(
                target_path.to_string_lossy().to_string(),
            ));
        }
        tokio::fs::read(&target_path).await.map_err(Into::into)
    }

    /// Write content to a file in the archive
    pub async fn write_file(
        &mut self,
        relative_path: impl AsRef<Path>,
        content: &[u8],
    ) -> Result<()> {
        let target_path = self.temp_path().join(relative_path.as_ref());

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&target_path, content).await?;

        // Add to files list if not already present
        if !self.files.contains(&target_path) {
            self.files.push(target_path);
            self.files.sort();
        }

        Ok(())
    }
}

/// Iterator implementation for ArchiveHandler
///
/// Iterates over all file paths in the extracted archive.
impl<'a> IntoIterator for &'a ArchiveHandler {
    type IntoIter = std::slice::Iter<'a, PathBuf>;
    type Item = &'a PathBuf;

    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}

impl IntoIterator for ArchiveHandler {
    type IntoIter = std::vec::IntoIter<PathBuf>;
    type Item = PathBuf;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_archive_handler_creation() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![PathBuf::from("test.txt")];

        let handler = ArchiveHandler::new(
            ArchiveType::Zip,
            Some(PathBuf::from("test.zip")),
            temp_dir,
            files.clone(),
        );

        assert_eq!(handler.archive_type, ArchiveType::Zip);
        assert_eq!(handler.file_count(), 1);
        assert!(!handler.is_empty());
    }

    #[test]
    fn test_empty_archive_handler() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![];

        let handler = ArchiveHandler::new(ArchiveType::Zip, None, temp_dir, files);

        assert_eq!(handler.file_count(), 0);
        assert!(handler.is_empty());
    }

    #[test]
    fn test_find_files_by_extension() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![
            PathBuf::from("test.txt"),
            PathBuf::from("data.json"),
            PathBuf::from("image.png"),
        ];

        let handler = ArchiveHandler::new(ArchiveType::Zip, None, temp_dir, files);

        let txt_files = handler.find_files_by_extension("txt");
        assert_eq!(txt_files.len(), 1);

        let json_files = handler.find_files_by_extension("json");
        assert_eq!(json_files.len(), 1);
    }

    #[test]
    fn test_iterator() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")];

        let handler = ArchiveHandler::new(ArchiveType::Zip, None, temp_dir, files.clone());

        let collected: Vec<&PathBuf> = (&handler).into_iter().collect();
        assert_eq!(collected.len(), 2);
    }

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let mut handler = ArchiveHandler::new(ArchiveType::Zip, None, temp_dir, vec![]);

        let content = b"Hello, World!";
        handler.write_file("test.txt", content).await.unwrap();

        assert!(handler.contains_file("test.txt"));
        let read_content = handler.read_file("test.txt").await.unwrap();
        assert_eq!(read_content, content);
    }
}
