//! TAR archive handler implementation
//!
//! This module provides specialized handling for TAR archives using the tar crate,
//! including support for compressed TAR formats (tar.gz, tar.bz2, tar.xz).

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use tar::{Archive, Builder, EntryType};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::{ArchiveType, Error, Result};

/// Buffered writer for XZ compression using liblzma-rs
///
/// This writer buffers all data and compresses it when dropped or explicitly finished.
struct XzBufferedWriter<W: Write> {
    writer: Option<W>,
    buffer: Vec<u8>,
}

impl<W: Write> XzBufferedWriter<W> {
    fn new(writer: W, _buffer: Vec<u8>) -> Self {
        Self {
            writer: Some(writer),
            buffer: Vec::new(),
        }
    }

    fn finish(&mut self) -> std::io::Result<()> {
        if let Some(writer) = self.writer.take() {
            use xz2::write::XzEncoder;
            let mut encoder = XzEncoder::new(writer, 6);
            encoder.write_all(&self.buffer)?;
            encoder.finish()?;
        }
        Ok(())
    }
}

impl<W: Write> Write for XzBufferedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // For buffered XZ compression, we don't flush until finish()
        Ok(())
    }
}

impl<W: Write> Drop for XzBufferedWriter<W> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

/// Specialized handler for TAR archive operations
///
/// This handler provides efficient TAR-specific operations using the tar crate,
/// with support for various compression formats.
pub struct TarArchiveHandler<R: Read> {
    /// The underlying TAR archive
    archive: Archive<R>,
    /// Archive type (for compression handling)
    archive_type: ArchiveType,
}

impl<R: Read> TarArchiveHandler<R> {
    /// Create a new TAR handler from a reader
    pub fn new(reader: R, archive_type: ArchiveType) -> Result<Self> {
        if !archive_type.is_tar_variant() {
            return Err(Error::unsupported_format(format!(
                "Expected TAR variant, got: {}",
                archive_type
            )));
        }

        Ok(Self {
            archive: Archive::new(reader),
            archive_type,
        })
    }

    /// Get the archive type
    pub fn archive_type(&self) -> ArchiveType {
        self.archive_type
    }

    /// Set whether to preserve permissions when extracting
    pub fn set_preserve_permissions(&mut self, preserve: bool) {
        self.archive.set_preserve_permissions(preserve);
    }

    /// Set whether to preserve modification times when extracting
    pub fn set_preserve_mtime(&mut self, preserve: bool) {
        self.archive.set_preserve_mtime(preserve);
    }

    /// Set whether to unpack extended attributes
    pub fn set_unpack_xattrs(&mut self, unpack: bool) {
        self.archive.set_unpack_xattrs(unpack);
    }

    /// Extract all entries to the specified directory
    pub async fn extract_to(&mut self, target_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let target_dir = target_dir.as_ref();
        fs::create_dir_all(target_dir).await?;

        let mut extracted_files = Vec::new();

        for entry in self.archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            let target_path = target_dir.join(&path);

            // Create parent directories
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            match entry.header().entry_type() {
                EntryType::Regular => {
                    let mut content = Vec::new();
                    entry.read_to_end(&mut content)?;

                    let mut file = fs::File::create(&target_path).await?;
                    file.write_all(&content).await?;

                    extracted_files.push(target_path);
                }
                EntryType::Directory => {
                    fs::create_dir_all(&target_path).await?;
                }
                EntryType::Symlink => {
                    if let Ok(Some(link_target)) = entry.link_name() {
                        #[cfg(unix)]
                        {
                            tokio::fs::symlink(&link_target, &target_path).await?;
                        }
                        #[cfg(windows)]
                        {
                            // Windows requires different handling for symlinks
                            if target_path.is_dir() {
                                tokio::fs::symlink_dir(&link_target, &target_path).await?;
                            } else {
                                tokio::fs::symlink_file(&link_target, &target_path).await?;
                            }
                        }
                    }
                }
                EntryType::Link => {
                    // Hard links - create a copy for simplicity
                    if let Ok(Some(link_target)) = entry.link_name() {
                        let source_path = target_dir.join(link_target);
                        if source_path.exists() {
                            fs::copy(&source_path, &target_path).await?;
                            extracted_files.push(target_path);
                        }
                    }
                }
                _ => {
                    // Handle other entry types as needed
                    // For now, we skip unsupported types
                }
            }
        }

        Ok(extracted_files)
    }

    /// Get entries as an iterator
    pub fn entries(&mut self) -> Result<tar::Entries<'_, R>> {
        Ok(self.archive.entries()?)
    }

    /// List all entries without extracting
    pub fn list_entries(&mut self) -> Result<Vec<TarEntryInfo>> {
        let mut entries = Vec::new();

        for entry in self.archive.entries()? {
            let entry = entry?;
            let header = entry.header();

            let info = TarEntryInfo {
                path: entry.path()?.to_path_buf(),
                size: header.size()?,
                entry_type: header.entry_type(),
                mode: header.mode()?,
                uid: header.uid()?,
                gid: header.gid()?,
                mtime: header.mtime()?,
            };

            entries.push(info);
        }

        Ok(entries)
    }
}

/// Information about a TAR entry
#[derive(Debug, Clone)]
pub struct TarEntryInfo {
    /// Path of the entry within the archive
    pub path: PathBuf,
    /// Size of the entry in bytes
    pub size: u64,
    /// Type of entry (file, directory, symlink, etc.)
    pub entry_type: EntryType,
    /// File mode/permissions
    pub mode: u32,
    /// User ID
    pub uid: u64,
    /// Group ID
    pub gid: u64,
    /// Modification time (Unix timestamp)
    pub mtime: u64,
}

/// Builder for creating TAR archives
pub struct TarArchiveBuilder<W: Write> {
    builder: Builder<W>,
    archive_type: ArchiveType,
}

impl<W: Write> TarArchiveBuilder<W> {
    /// Create a new TAR archive builder
    pub fn new(writer: W, archive_type: ArchiveType) -> Result<Self> {
        if !archive_type.is_tar_variant() {
            return Err(Error::unsupported_format(format!(
                "Expected TAR variant, got: {}",
                archive_type
            )));
        }

        Ok(Self {
            builder: Builder::new(writer),
            archive_type,
        })
    }

    /// Get the archive type
    pub fn archive_type(&self) -> ArchiveType {
        self.archive_type
    }

    /// Add a file to the archive from a path
    pub fn append_path_with_name<P: AsRef<Path>, N: AsRef<Path>>(
        &mut self,
        path: P,
        name: N,
    ) -> Result<()> {
        self.builder.append_path_with_name(path, name)?;
        Ok(())
    }

    /// Add a file to the archive with the same name as the path
    pub fn append_path<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.builder.append_path(path)?;
        Ok(())
    }

    /// Add a directory to the archive
    pub fn append_dir<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        path: P,
        src_path: Q,
    ) -> Result<()> {
        self.builder.append_dir(path, src_path)?;
        Ok(())
    }

    /// Add a directory recursively to the archive
    pub fn append_dir_all<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        path: P,
        src_path: Q,
    ) -> Result<()> {
        self.builder.append_dir_all(path, src_path)?;
        Ok(())
    }

    /// Add data from a reader to the archive
    pub fn append_data<P: AsRef<Path>, R: Read>(
        &mut self,
        path: P,
        size: u64,
        data: R,
    ) -> Result<()> {
        let mut header = tar::Header::new_gnu();
        header.set_size(size);
        header.set_mode(0o644);
        header.set_cksum();

        self.builder.append_data(&mut header, path, data)?;
        Ok(())
    }

    /// Finish writing the archive
    pub fn finish(self) -> Result<W> {
        Ok(self.builder.into_inner()?)
    }
}

/// Static methods for creating archives from directories
impl TarArchiveBuilder<std::fs::File> {
    /// Create a new TAR archive builder for creating from directory
    pub fn for_directory(archive_type: ArchiveType) -> Self {
        // This is a placeholder - we'll create the actual file in create_from_directory
        Self {
            builder: Builder::new(tempfile::tempfile().expect("Failed to create temp file")),
            archive_type,
        }
    }

    /// Create a TAR archive from a directory
    pub async fn create_from_directory(self, source_dir: &Path, target_path: &Path) -> Result<()> {
        use std::fs;

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

        match self.archive_type {
            ArchiveType::Tar => {
                let file = std::fs::File::create(target_path)?;
                let mut builder = Builder::new(file);

                for file_path in files {
                    let relative_path = file_path.strip_prefix(source_dir).map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid file path: {}", e),
                        )
                    })?;
                    builder.append_path_with_name(&file_path, relative_path)?;
                }

                builder.finish()?;
            }
            ArchiveType::TarGz => {
                use flate2::Compression;
                use flate2::write::GzEncoder;

                let file = std::fs::File::create(target_path)?;
                let encoder = GzEncoder::new(file, Compression::default());
                let mut builder = Builder::new(encoder);

                for file_path in files {
                    let relative_path = file_path.strip_prefix(source_dir).map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid file path: {}", e),
                        )
                    })?;
                    builder.append_path_with_name(&file_path, relative_path)?;
                }

                builder.finish()?;
            }
            ArchiveType::TarBz2 => {
                use bzip2::Compression;
                use bzip2::write::BzEncoder;

                let file = std::fs::File::create(target_path)?;
                let encoder = BzEncoder::new(file, Compression::default());
                let mut builder = Builder::new(encoder);

                for file_path in files {
                    let relative_path = file_path.strip_prefix(source_dir).map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid file path: {}", e),
                        )
                    })?;
                    builder.append_path_with_name(&file_path, relative_path)?;
                }

                builder.finish()?;
            }
            ArchiveType::TarXz => {
                use xz2::write::XzEncoder;

                let file = std::fs::File::create(target_path)?;
                let encoder = XzEncoder::new(file, 6);
                let mut builder = Builder::new(encoder);

                for file_path in files {
                    let relative_path = file_path.strip_prefix(source_dir).map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid file path: {}", e),
                        )
                    })?;
                    builder.append_path_with_name(&file_path, relative_path)?;
                }

                let encoder = builder.into_inner()?;
                encoder.finish()?;
            }
            _ => {
                return Err(Error::unsupported_format(format!(
                    "Unsupported TAR variant: {}",
                    self.archive_type
                )));
            }
        }

        Ok(())
    }
}

/// Convenience functions for creating compressed TAR handlers
impl TarArchiveHandler<Cursor<Vec<u8>>> {
    /// Create a TAR handler from compressed data
    pub fn from_compressed_data(
        data: Vec<u8>,
        archive_type: ArchiveType,
    ) -> Result<TarArchiveHandler<Box<dyn Read>>> {
        let cursor = Cursor::new(data);

        match archive_type {
            ArchiveType::Tar => {
                let reader: Box<dyn Read> = Box::new(cursor);
                Ok(TarArchiveHandler {
                    archive: Archive::new(reader),
                    archive_type,
                })
            }
            ArchiveType::TarGz => {
                use flate2::read::GzDecoder;
                let decoder = GzDecoder::new(cursor);
                let reader: Box<dyn Read> = Box::new(decoder);
                Ok(TarArchiveHandler {
                    archive: Archive::new(reader),
                    archive_type,
                })
            }
            ArchiveType::TarBz2 => {
                use bzip2::read::BzDecoder;
                let decoder = BzDecoder::new(cursor);
                let reader: Box<dyn Read> = Box::new(decoder);
                Ok(TarArchiveHandler {
                    archive: Archive::new(reader),
                    archive_type,
                })
            }
            ArchiveType::TarXz => {
                use xz2::read::XzDecoder;
                let decoder = XzDecoder::new(cursor);
                let reader: Box<dyn Read> = Box::new(decoder);
                Ok(TarArchiveHandler {
                    archive: Archive::new(reader),
                    archive_type,
                })
            }
            _ => Err(Error::unsupported_format(format!(
                "Not a TAR variant: {}",
                archive_type
            ))),
        }
    }
}

/// Convenience functions for creating compressed TAR builders
impl<W: Write + Send + 'static> TarArchiveBuilder<W> {
    /// Create a compressed TAR builder
    pub fn compressed(
        writer: W,
        archive_type: ArchiveType,
    ) -> Result<TarArchiveBuilder<Box<dyn Write + Send>>> {
        match archive_type {
            ArchiveType::Tar => {
                let writer: Box<dyn Write + Send> = Box::new(writer);
                Ok(TarArchiveBuilder {
                    builder: Builder::new(writer),
                    archive_type,
                })
            }
            ArchiveType::TarGz => {
                use flate2::Compression;
                use flate2::write::GzEncoder;
                let encoder = GzEncoder::new(writer, Compression::default());
                let writer: Box<dyn Write + Send> = Box::new(encoder);
                Ok(TarArchiveBuilder {
                    builder: Builder::new(writer),
                    archive_type,
                })
            }
            ArchiveType::TarBz2 => {
                use bzip2::Compression;
                use bzip2::write::BzEncoder;
                let encoder = BzEncoder::new(writer, Compression::default());
                let writer: Box<dyn Write + Send> = Box::new(encoder);
                Ok(TarArchiveBuilder {
                    builder: Builder::new(writer),
                    archive_type,
                })
            }
            ArchiveType::TarXz => {
                // For XZ compression, we need to buffer the data and compress it at the end
                // This is a limitation of liblzma-rs compared to xz2's streaming interface
                let buffer = Vec::new();
                let xz_writer = XzBufferedWriter::new(writer, buffer);
                let writer: Box<dyn Write + Send> = Box::new(xz_writer);
                Ok(TarArchiveBuilder {
                    builder: Builder::new(writer),
                    archive_type,
                })
            }
            _ => Err(Error::unsupported_format(format!(
                "Not a TAR variant: {}",
                archive_type
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[tokio::test]
    async fn test_tar_handler_creation() {
        let data = Vec::new();
        let cursor = Cursor::new(data);
        let handler = TarArchiveHandler::new(cursor, ArchiveType::Tar);
        assert!(handler.is_ok());
    }

    #[test]
    fn test_tar_handler_invalid_type() {
        let data = Vec::new();
        let cursor = Cursor::new(data);
        let handler = TarArchiveHandler::new(cursor, ArchiveType::Zip);
        assert!(handler.is_err());
    }

    #[test]
    fn test_tar_builder_creation() {
        let writer = Vec::new();
        let builder = TarArchiveBuilder::new(writer, ArchiveType::Tar);
        assert!(builder.is_ok());
    }

    #[test]
    fn test_compressed_builder_creation() {
        let writer = Vec::new();
        let builder = TarArchiveBuilder::compressed(writer, ArchiveType::TarGz);
        assert!(builder.is_ok());
    }

    #[test]
    fn test_entry_info() {
        let info = TarEntryInfo {
            path: PathBuf::from("test.txt"),
            size: 100,
            entry_type: EntryType::Regular,
            mode: 0o644,
            uid: 1000,
            gid: 1000,
            mtime: 1234567890,
        };

        assert_eq!(info.path, PathBuf::from("test.txt"));
        assert_eq!(info.size, 100);
        assert_eq!(info.mode, 0o644);
    }
}
