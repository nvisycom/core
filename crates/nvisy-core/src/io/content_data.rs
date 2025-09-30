//! Content data structure for storing and managing content with metadata
//!
//! This module provides the [`ContentData`] struct for storing content data
//! along with its metadata and source information.

use std::fmt;
use std::sync::Mutex;

use bytes::Bytes;
use nvisy_error::{Error, ErrorResource, ErrorType, Result};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::path::ContentSource;

/// Content data with metadata and computed hashes
///
/// This struct is a minimal wrapper around `bytes::Bytes` that stores content data
/// along with metadata about its source and optional computed SHA256 hash.
/// It's designed to be cheap to clone using the `bytes::Bytes` type.
/// The SHA256 hash is protected by a mutex for thread safety.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ContentData {
    /// Unique identifier for the content source
    pub content_source: ContentSource,
    /// The actual content data
    pub content_data: Bytes,
    /// Optional SHA256 hash of the content as bytes, protected by mutex
    #[cfg_attr(feature = "serde", serde(skip))]
    content_sha256: Mutex<Option<Bytes>>,
}

impl ContentData {
    /// Create new content data
    ///
    /// # Example
    ///
    /// ```
    /// use nvisy_core::{io::ContentData, ContentSource};
    /// use bytes::Bytes;
    ///
    /// let source = ContentSource::new();
    /// let data = Bytes::from("Hello, world!");
    /// let content = ContentData::new(source, data);
    ///
    /// assert_eq!(content.size(), 13);
    /// ```
    pub fn new(content_source: ContentSource, content_data: Bytes) -> Self {
        Self {
            content_source,
            content_data,
            content_sha256: Mutex::new(None),
        }
    }

    /// Get the size of the content in bytes
    pub fn size(&self) -> usize {
        self.content_data.len()
    }

    /// Get pretty formatted size string
    pub fn get_pretty_size(&self) -> String {
        let bytes = self.size();
        match bytes {
            0..=1023 => format!("{} B", bytes),
            1024..=1048575 => format!("{:.1} KB", bytes as f64 / 1024.0),
            1048576..=1073741823 => format!("{:.1} MB", bytes as f64 / 1048576.0),
            _ => format!("{:.1} GB", bytes as f64 / 1073741824.0),
        }
    }

    /// Get the content data as bytes slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.content_data
    }

    /// Get the content data as bytes
    pub fn into_bytes(self) -> Bytes {
        self.content_data
    }

    /// Check if the content is likely text (basic heuristic)
    pub fn is_likely_text(&self) -> bool {
        self.content_data
            .iter()
            .all(|&b| b.is_ascii_graphic() || b.is_ascii_whitespace())
    }

    /// Try to convert the content data to a UTF-8 string
    pub fn as_string(&self) -> Result<String> {
        String::from_utf8(self.content_data.to_vec()).map_err(|e| {
            Error::new(
                ErrorType::Runtime,
                ErrorResource::Core,
                format!("Invalid UTF-8: {}", e),
            )
        })
    }

    /// Try to convert the content data to a UTF-8 string slice
    pub fn as_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.content_data).map_err(|e| {
            Error::new(
                ErrorType::Runtime,
                ErrorResource::Core,
                format!("Invalid UTF-8: {}", e),
            )
        })
    }

    /// Compute and store SHA256 hash of the content, returning the hash as bytes
    pub fn compute_sha256(&self) -> Bytes {
        let mut hasher = Sha256::new();
        hasher.update(&self.content_data);
        let hash_bytes = Bytes::from(hasher.finalize().to_vec());

        if let Ok(mut guard) = self.content_sha256.lock() {
            *guard = Some(hash_bytes.clone());
        }

        hash_bytes
    }

    /// Get the SHA256 hash if computed, computing it if not already done
    pub fn sha256(&self) -> Bytes {
        if let Ok(guard) = self.content_sha256.lock() {
            if let Some(ref hash) = *guard {
                return hash.clone();
            }
        }
        self.compute_sha256()
    }

    /// Get the SHA256 hash as hex string
    pub fn sha256_hex(&self) -> String {
        hex::encode(self.sha256())
    }

    /// Verify the content against a provided SHA256 hash
    pub fn verify_sha256(&self, expected_hash: impl AsRef<[u8]>) -> Result<()> {
        let actual_hash = self.sha256();
        let expected = expected_hash.as_ref();

        if actual_hash.as_ref() == expected {
            Ok(())
        } else {
            Err(Error::new(
                ErrorType::Runtime,
                ErrorResource::Core,
                format!(
                    "Hash mismatch: expected {}, got {}",
                    hex::encode(expected),
                    hex::encode(&actual_hash)
                ),
            ))
        }
    }

    /// Get a slice of the content data
    pub fn slice(&self, start: usize, end: usize) -> Result<Bytes> {
        if end > self.content_data.len() {
            return Err(Error::new(
                ErrorType::Runtime,
                ErrorResource::Core,
                format!(
                    "Slice end {} exceeds content length {}",
                    end,
                    self.content_data.len()
                ),
            ));
        }
        if start > end {
            return Err(Error::new(
                ErrorType::Runtime,
                ErrorResource::Core,
                format!("Slice start {} is greater than end {}", start, end),
            ));
        }
        Ok(self.content_data.slice(start..end))
    }

    /// Check if the content is empty
    pub fn is_empty(&self) -> bool {
        self.content_data.is_empty()
    }
}

// Manual implementation of Clone since Mutex doesn't implement Clone
impl Clone for ContentData {
    fn clone(&self) -> Self {
        let hash = if let Ok(guard) = self.content_sha256.lock() {
            guard.clone()
        } else {
            None
        };

        Self {
            content_source: self.content_source,
            content_data: self.content_data.clone(),
            content_sha256: Mutex::new(hash),
        }
    }
}

// Manual implementation of PartialEq since Mutex doesn't implement PartialEq
impl PartialEq for ContentData {
    fn eq(&self, other: &Self) -> bool {
        if self.content_source != other.content_source || self.content_data != other.content_data {
            return false;
        }

        // Compare hashes if both are computed
        let self_hash = if let Ok(guard) = self.content_sha256.lock() {
            guard.clone()
        } else {
            None
        };

        let other_hash = if let Ok(guard) = other.content_sha256.lock() {
            guard.clone()
        } else {
            None
        };

        self_hash == other_hash
    }
}

impl Eq for ContentData {}

// Implement From conversions for common types
impl From<&str> for ContentData {
    fn from(s: &str) -> Self {
        let source = ContentSource::new();
        Self::new(source, Bytes::from(s.to_string()))
    }
}

impl From<String> for ContentData {
    fn from(s: String) -> Self {
        let source = ContentSource::new();
        Self::new(source, Bytes::from(s))
    }
}

impl From<&[u8]> for ContentData {
    fn from(bytes: &[u8]) -> Self {
        let source = ContentSource::new();
        Self::new(source, Bytes::copy_from_slice(bytes))
    }
}

impl From<Vec<u8>> for ContentData {
    fn from(vec: Vec<u8>) -> Self {
        let source = ContentSource::new();
        Self::new(source, Bytes::from(vec))
    }
}

impl From<Bytes> for ContentData {
    fn from(bytes: Bytes) -> Self {
        let source = ContentSource::new();
        Self::new(source, bytes)
    }
}

impl fmt::Display for ContentData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(text) = self.as_str() {
            write!(f, "{}", text)
        } else {
            write!(f, "[Binary data: {} bytes]", self.size())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_data_creation() {
        let source = ContentSource::new();
        let data = Bytes::from("Hello, world!");
        let content = ContentData::new(source, data);

        assert_eq!(content.content_source, source);
        assert_eq!(content.size(), 13);
        // Check that hash is not computed yet
        assert!(content.content_sha256.lock().unwrap().is_none());
    }

    #[test]
    fn test_size_methods() {
        let content = ContentData::from("Hello");
        assert_eq!(content.size(), 5);

        let pretty_size = content.get_pretty_size();
        assert!(!pretty_size.is_empty());
    }

    #[test]
    fn test_sha256_computation() {
        let content = ContentData::from("Hello, world!");
        let hash = content.compute_sha256();

        assert!(content.content_sha256.lock().unwrap().is_some());
        assert_eq!(hash.len(), 32); // SHA256 is 32 bytes

        // Test getting cached hash
        let hash2 = content.sha256();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_sha256_verification() {
        let content = ContentData::from("Hello, world!");
        let hash = content.compute_sha256();

        // Should verify successfully against itself
        assert!(content.verify_sha256(&hash).is_ok());

        // Should fail against different hash
        let wrong_hash = vec![0u8; 32];
        assert!(content.verify_sha256(&wrong_hash).is_err());
    }

    #[test]
    fn test_string_conversion() {
        let content = ContentData::from("Hello, world!");
        assert_eq!(content.as_string().unwrap(), "Hello, world!");
        assert_eq!(content.as_str().unwrap(), "Hello, world!");

        let binary_content = ContentData::from(vec![0xFF, 0xFE, 0xFD]);
        assert!(binary_content.as_string().is_err());
        assert!(binary_content.as_str().is_err());
    }

    #[test]
    fn test_is_likely_text() {
        let text_content = ContentData::from("Hello, world!");
        assert!(text_content.is_likely_text());

        let binary_content = ContentData::from(vec![0xFF, 0xFE, 0xFD]);
        assert!(!binary_content.is_likely_text());
    }

    #[test]
    fn test_slice() {
        let content = ContentData::from("Hello, world!");

        let slice = content.slice(0, 5).unwrap();
        assert_eq!(slice, Bytes::from("Hello"));

        let slice = content.slice(7, 12).unwrap();
        assert_eq!(slice, Bytes::from("world"));

        // Test bounds checking
        assert!(content.slice(0, 100).is_err());
        assert!(content.slice(10, 5).is_err());
    }

    #[test]
    fn test_from_conversions() {
        let from_str = ContentData::from("test");
        let from_string = ContentData::from("test".to_string());
        let from_bytes = ContentData::from(b"test".as_slice());
        let from_vec = ContentData::from(b"test".to_vec());
        let from_bytes_type = ContentData::from(Bytes::from("test"));

        assert_eq!(from_str.as_str().unwrap(), "test");
        assert_eq!(from_string.as_str().unwrap(), "test");
        assert_eq!(from_bytes.as_str().unwrap(), "test");
        assert_eq!(from_vec.as_str().unwrap(), "test");
        assert_eq!(from_bytes_type.as_str().unwrap(), "test");
    }

    #[test]
    fn test_display() {
        let text_content = ContentData::from("Hello");
        assert_eq!(format!("{}", text_content), "Hello");

        let binary_content = ContentData::from(vec![0xFF, 0xFE]);
        assert!(format!("{}", binary_content).contains("Binary data"));
    }

    #[test]
    fn test_cloning_is_cheap() {
        let original = ContentData::from("Hello, world!");
        let cloned = original.clone();

        // They should be equal
        assert_eq!(original, cloned);

        // But the underlying bytes should share the same memory
        assert_eq!(original.content_data.as_ptr(), cloned.content_data.as_ptr());
    }

    #[test]
    fn test_into_bytes() {
        let content = ContentData::from("Hello, world!");
        let bytes = content.into_bytes();
        assert_eq!(bytes, Bytes::from("Hello, world!"));
    }

    #[test]
    fn test_empty_content() {
        let content = ContentData::from("");
        assert!(content.is_empty());
        assert_eq!(content.size(), 0);
    }
}
