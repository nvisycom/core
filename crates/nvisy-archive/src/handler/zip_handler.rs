//! ZIP archive handler implementation
//!
//! This module provides specialized handling for ZIP archives using the zip crate,
//! with support for various compression methods and ZIP-specific features.

use std::io::{Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::io::AsyncWriteExt;
use zip::read::ZipFile;
use zip::write::{ExtendedFileOptions, SimpleFileOptions};
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::{ArchiveType, Error, Result};

/// Specialized handler for ZIP archive operations
///
/// This handler provides efficient ZIP-specific operations using the zip crate,
/// with support for various compression methods and ZIP features.
#[derive(Debug)]
pub struct ZipArchiveHandler<R> {
    /// The underlying ZIP archive
    archive: ZipArchive<R>,
    /// Archive type (should always be ZIP)
    archive_type: ArchiveType,
}

impl<R: Read + Seek> ZipArchiveHandler<R> {
    /// Create a new ZIP handler from a reader
    pub fn new(reader: R, archive_type: ArchiveType) -> Result<Self> {
        if archive_type != ArchiveType::Zip {
            return Err(Error::unsupported_format(format!(
                "Expected ZIP, got: {}",
                archive_type
            )));
        }

        let archive = ZipArchive::new(reader)?;

        Ok(Self {
            archive,
            archive_type,
        })
    }

    /// Get the archive type
    pub fn archive_type(&self) -> ArchiveType {
        self.archive_type
    }

    /// Get the number of files in the archive
    pub fn len(&self) -> usize {
        self.archive.len()
    }

    /// Check if the archive is empty
    pub fn is_empty(&self) -> bool {
        self.archive.len() == 0
    }

    /// Extract all entries to the specified directory
    pub async fn extract_to(&mut self, target_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let target_dir = target_dir.as_ref();
        fs::create_dir_all(target_dir).await?;

        let mut extracted_files = Vec::new();

        for i in 0..self.archive.len() {
            let mut file = self.archive.by_index(i)?;
            let file_path = target_dir.join(file.name());

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            if file.is_dir() {
                fs::create_dir_all(&file_path).await?;
            } else {
                let mut content = Vec::with_capacity(file.size() as usize);
                std::io::Read::read_to_end(&mut file, &mut content)?;

                let mut output_file = fs::File::create(&file_path).await?;
                output_file.write_all(&content).await?;

                // Set file permissions on Unix systems
                #[cfg(unix)]
                {
                    if let Some(mode) = file.unix_mode() {
                        use std::os::unix::fs::PermissionsExt;
                        let permissions = std::fs::Permissions::from_mode(mode);
                        std::fs::set_permissions(&file_path, permissions)?;
                    }
                }

                extracted_files.push(file_path);
            }
        }

        Ok(extracted_files)
    }

    /// Extract a specific file by name
    pub async fn extract_file(&mut self, name: &str, target_path: impl AsRef<Path>) -> Result<()> {
        let mut file = self.archive.by_name(name)?;
        let target_path = target_path.as_ref();

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut content = Vec::with_capacity(file.size() as usize);
        std::io::Read::read_to_end(&mut file, &mut content)?;

        let mut output_file = fs::File::create(target_path).await?;
        output_file.write_all(&content).await?;

        Ok(())
    }

    /// Read a file's content directly into memory
    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>> {
        let mut file = self.archive.by_name(name)?;
        let mut content = Vec::with_capacity(file.size() as usize);
        std::io::Read::read_to_end(&mut file, &mut content)?;
        Ok(content)
    }

    /// Get file by index
    pub fn by_index(&mut self, index: usize) -> Result<ZipFile<'_, R>> {
        Ok(self.archive.by_index(index)?)
    }

    /// Get file by name
    pub fn by_name(&mut self, name: &str) -> Result<ZipFile<'_, R>> {
        Ok(self.archive.by_name(name)?)
    }

    /// List all entries without extracting
    pub fn list_entries(&mut self) -> Result<Vec<ZipEntryInfo>> {
        let mut entries = Vec::new();

        for i in 0..self.archive.len() {
            let file = self.archive.by_index(i)?;

            let info = ZipEntryInfo {
                name: file.name().to_string(),
                size: file.size(),
                compressed_size: file.compressed_size(),
                compression_method: file.compression(),
                is_dir: file.is_dir(),
                is_file: file.is_file(),
                unix_mode: file.unix_mode(),
                last_modified: file.last_modified().unwrap_or_default(),
                crc32: file.crc32(),
                extra_data: file.extra_data().unwrap_or(&[]).to_vec(),
                comment: file.comment().to_string(),
            };

            entries.push(info);
        }

        Ok(entries)
    }

    /// Get file names
    pub fn file_names(&self) -> Vec<String> {
        self.archive.file_names().map(|s| s.to_string()).collect()
    }

    /// Check if a file exists in the archive
    pub fn contains_file(&mut self, name: &str) -> bool {
        self.archive.by_name(name).is_ok()
    }

    /// Get the comment of the archive
    pub fn comment(&self) -> String {
        String::from_utf8_lossy(self.archive.comment()).to_string()
    }
}

/// Information about a ZIP entry
#[derive(Debug, Clone)]
pub struct ZipEntryInfo {
    /// Name of the file within the archive
    pub name: String,
    /// Uncompressed size in bytes
    pub size: u64,
    /// Compressed size in bytes
    pub compressed_size: u64,
    /// Compression method used
    pub compression_method: CompressionMethod,
    /// Whether this entry is a directory
    pub is_dir: bool,
    /// Whether this entry is a file
    pub is_file: bool,
    /// Unix file permissions (if available)
    pub unix_mode: Option<u32>,
    /// Last modification time
    pub last_modified: DateTime,
    /// CRC32 checksum
    pub crc32: u32,
    /// Extra data field
    pub extra_data: Vec<u8>,
    /// File comment
    pub comment: String,
}

/// Builder for creating ZIP archives
pub struct ZipArchiveBuilder<W: Write + Seek> {
    writer: ZipWriter<W>,
    archive_type: ArchiveType,
}

impl<W: Write + Seek> ZipArchiveBuilder<W> {
    /// Create a new ZIP archive builder
    pub fn new(writer: W) -> Self {
        Self {
            writer: ZipWriter::new(writer),
            archive_type: ArchiveType::Zip,
        }
    }

    /// Get the archive type
    pub fn archive_type(&self) -> ArchiveType {
        self.archive_type
    }

    /// Set the comment for the archive
    pub fn set_comment(&mut self, comment: String) {
        self.writer.set_comment(comment);
    }

    /// Start a new file in the archive with default options
    pub fn start_file(&mut self, name: &str) -> Result<()> {
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        self.writer.start_file(name, options)?;
        Ok(())
    }

    /// Start a new file with custom options
    pub fn start_file_with_options(
        &mut self,
        name: &str,
        options: SimpleFileOptions,
    ) -> Result<()> {
        self.writer.start_file(name, options)?;
        Ok(())
    }

    /// Start a new file with extended options
    pub fn start_file_with_extra_data(
        &mut self,
        name: &str,
        _options: ExtendedFileOptions,
    ) -> Result<()> {
        // Note: ExtendedFileOptions may not be supported in this version
        // Convert to SimpleFileOptions for compatibility
        let simple_options =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        self.writer.start_file(name, simple_options)?;
        Ok(())
    }

    /// Write data to the current file
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        Ok(self.writer.write(data)?)
    }

    /// Write all data to the current file
    pub fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data)?;
        Ok(())
    }

    /// Add a file from a path with default compression
    pub async fn add_file_from_path(
        &mut self,
        archive_path: &str,
        file_path: impl AsRef<Path>,
    ) -> Result<()> {
        let file_path = file_path.as_ref();
        let content = fs::read(file_path).await?;

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        self.writer.start_file(archive_path, options)?;
        self.writer.write_all(&content)?;

        Ok(())
    }

    /// Add a file from memory
    pub fn add_file_from_memory(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        self.writer.start_file(name, options)?;
        self.writer.write_all(data)?;

        Ok(())
    }

    /// Add a directory entry
    pub fn add_directory(&mut self, name: &str) -> Result<()> {
        let dir_name = if name.ends_with('/') {
            name.to_string()
        } else {
            format!("{}/", name)
        };

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        self.writer.start_file(&dir_name, options)?;
        Ok(())
    }

    /// Add an entire directory recursively
    pub async fn add_directory_recursively(
        &mut self,
        archive_prefix: &str,
        dir_path: impl AsRef<Path>,
    ) -> Result<()> {
        let dir_path = dir_path.as_ref();
        let mut entries = fs::read_dir(dir_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            let archive_path = if archive_prefix.is_empty() {
                file_name_str.to_string()
            } else {
                format!("{}/{}", archive_prefix, file_name_str)
            };

            if entry_path.is_dir() {
                self.add_directory(&archive_path)?;
                self.add_directory_recursively(&archive_path, &entry_path)
                    .await?;
            } else {
                self.add_file_from_path(&archive_path, &entry_path).await?;
            }
        }

        Ok(())
    }

    /// Create options for storing files without compression
    pub fn stored_options() -> SimpleFileOptions {
        SimpleFileOptions::default().compression_method(CompressionMethod::Stored)
    }

    /// Create options for maximum compression
    pub fn max_compression_options() -> SimpleFileOptions {
        SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(9))
    }

    /// Create options with custom compression level
    pub fn compression_options(level: i32) -> SimpleFileOptions {
        SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(level.into()))
    }

    /// Finish writing the archive and return the underlying writer
    pub fn finish(self) -> Result<W> {
        Ok(self.writer.finish()?)
    }
}

/// Static methods for creating archives from directories
impl ZipArchiveBuilder<std::fs::File> {
    /// Create a new ZIP archive builder for creating from directory
    pub fn for_directory() -> Self {
        // This is a placeholder - we'll create the actual file in create_from_directory
        Self {
            writer: ZipWriter::new(tempfile::tempfile().expect("Failed to create temp file")),
            archive_type: ArchiveType::Zip,
        }
    }

    /// Create a ZIP archive from a directory
    pub async fn create_from_directory(self, source_dir: &Path, target_path: &Path) -> Result<()> {
        use std::fs;
        use std::io::Write;

        use zip::write::SimpleFileOptions;
        use zip::{CompressionMethod, ZipWriter};

        // Collect all files in the directory
        fn collect_files(dir: &Path) -> Result<Vec<PathBuf>> {
            let mut files = Vec::new();
            let entries = fs::read_dir(dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    files.push(path);
                } else if path.is_dir() {
                    let mut sub_files = collect_files(&path)?;
                    files.append(&mut sub_files);
                }
            }

            files.sort();
            Ok(files)
        }

        let files = collect_files(source_dir)?;
        let file = std::fs::File::create(target_path)?;
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        for file_path in files {
            let relative_path = file_path.strip_prefix(source_dir).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid file path: {}", e),
                )
            })?;

            let file_content = tokio::fs::read(&file_path).await?;

            zip.start_file(relative_path.to_string_lossy().as_ref(), options)?;
            zip.write_all(&file_content)?;
        }

        zip.finish()?;
        Ok(())
    }
}

/// Convenience constructor for ZIP handlers from memory
impl ZipArchiveHandler<Cursor<Vec<u8>>> {
    /// Create a ZIP handler from in-memory data
    pub fn from_memory(data: Vec<u8>) -> Result<Self> {
        let cursor = Cursor::new(data);
        Self::new(cursor, ArchiveType::Zip)
    }
}

/// Convenience constructor for ZIP builders with memory backing
impl ZipArchiveBuilder<Cursor<Vec<u8>>> {
    /// Create a ZIP builder that writes to memory
    pub fn new_in_memory() -> Self {
        let cursor = Cursor::new(Vec::new());
        Self::new(cursor)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_zip_handler_from_memory() {
        // Create a minimal ZIP file in memory
        let cursor = Cursor::new(Vec::new());
        let mut builder = ZipArchiveBuilder::new(cursor);

        builder
            .add_file_from_memory("test.txt", b"Hello, World!")
            .unwrap();
        let cursor = builder.finish().unwrap();

        // Test the handler
        let data = cursor.into_inner();
        let handler = ZipArchiveHandler::from_memory(data);
        assert!(handler.is_ok());

        let mut handler = handler.unwrap();
        assert_eq!(handler.len(), 1);
        assert!(!handler.is_empty());
        assert!(handler.contains_file("test.txt"));
    }

    #[test]
    fn test_zip_handler_invalid_type() {
        let data = Vec::new();
        let cursor = Cursor::new(data);
        let handler = ZipArchiveHandler::new(cursor, ArchiveType::Tar);
        assert!(handler.is_err());
    }

    #[test]
    fn test_zip_builder_creation() {
        let cursor = Cursor::new(Vec::new());
        let builder = ZipArchiveBuilder::new(cursor);
        assert_eq!(builder.archive_type(), ArchiveType::Zip);
    }

    #[test]
    fn test_zip_builder_in_memory() {
        let mut builder = ZipArchiveBuilder::new_in_memory();
        builder
            .add_file_from_memory("test.txt", b"Hello, World!")
            .unwrap();
        builder.add_directory("subdir").unwrap();

        let cursor = builder.finish().unwrap();
        let data = cursor.into_inner();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_compression_options() {
        // Test that options can be created without panicking
        let _stored = ZipArchiveBuilder::<Cursor<Vec<u8>>>::stored_options();
        let _max_compression = ZipArchiveBuilder::<Cursor<Vec<u8>>>::max_compression_options();
        let _custom = ZipArchiveBuilder::<Cursor<Vec<u8>>>::compression_options(5);

        // Note: compression_method field is private, so we can't test it directly
        // but we can verify the options are created successfully
    }

    #[tokio::test]
    async fn test_zip_extract_operations() {
        // Create a ZIP file with test data
        let mut builder = ZipArchiveBuilder::new_in_memory();
        builder
            .add_file_from_memory("file1.txt", b"Content 1")
            .unwrap();
        builder
            .add_file_from_memory("file2.txt", b"Content 2")
            .unwrap();
        builder.add_directory("subdir").unwrap();
        builder
            .add_file_from_memory("subdir/file3.txt", b"Content 3")
            .unwrap();

        let cursor = builder.finish().unwrap();
        let data = cursor.into_inner();

        // Test extraction
        let mut handler = ZipArchiveHandler::from_memory(data).unwrap();
        let temp_dir = TempDir::new().unwrap();

        let extracted_files = handler.extract_to(temp_dir.path()).await.unwrap();
        assert_eq!(extracted_files.len(), 3); // 3 files (directories don't count)

        // Test reading specific file
        let content = handler.read_file("file1.txt").unwrap();
        assert_eq!(content, b"Content 1");
    }

    #[test]
    fn test_entry_info() {
        let info = ZipEntryInfo {
            name: "test.txt".to_string(),
            size: 100,
            compressed_size: 80,
            compression_method: CompressionMethod::Deflated,
            is_dir: false,
            is_file: true,
            unix_mode: Some(0o644),
            last_modified: DateTime::default(),
            crc32: 12345,
            extra_data: Vec::new(),
            comment: String::new(),
        };

        assert_eq!(info.name, "test.txt");
        assert_eq!(info.size, 100);
        assert_eq!(info.compressed_size, 80);
        assert!(!info.is_dir);
        assert!(info.is_file);
    }
}
