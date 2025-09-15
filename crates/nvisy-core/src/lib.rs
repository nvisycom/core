#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

//! # Nvisy Core
//!
//! Core types and enums for data categorization in the Nvisy content processing system.
//!
//! This crate provides the fundamental data classification system used throughout
//! the Nvisy ecosystem to identify and categorize different types of sensitive data.
//!
//! ## Features
//!
//! - `serde`: Enable serialization support with serde
//!
//! ## Core Types
//!
//! - [`DataSensitivity`]: Sensitivity levels for risk assessment (in `fs` module)
//! - [`Content`]: Content types and data structures (in `io` module)
//! - [`DataReference`]: Data references with source tracking (in `io` module)
//! - [`DataStructureKind`]: Classification of data structure types (in `fs` module)
//! - [`ContentFile`]: File operations with content tracking (in `fs` module)
//! - [`ContentData`]: Container for content data with metadata (in `io` module)
//!
//! [ContentMetadata]: fs::ContentMetadata
//! [ContentFile]: fs::ContentFile
//! [ContentKind]: fs::ContentKind
//! [DataSensitivity]: fs::DataSensitivity
//! [DataStructureKind]: fs::DataStructureKind
//! [SupportedFormat]: fs::SupportedFormat
//! [Content]: io::Content
//! [ContentData]: io::ContentData
//! [DataReference]: io::DataReference
//! [ContentSource]: path::ContentSource

pub mod fs;
pub mod io;
pub mod path;
