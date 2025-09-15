# nvisy.com/core

[![Build Status][action-badge]][action-url]
[![Crate Docs][docs-badge]][docs-url]
[![Crate Version][crates-badge]][crates-url]
[![Crate Coverage][coverage-badge]][coverage-url]

**Check out other `nvisy` projects [here](https://github.com/nvisycom).**

[action-badge]: https://img.shields.io/github/actions/workflow/status/nvisycom/core/build.yaml?branch=main&label=build&logo=github&style=flat-square
[action-url]: https://github.com/nvisycom/core/actions/workflows/build.yaml
[crates-badge]: https://img.shields.io/crates/v/nvisy-core.svg?logo=rust&style=flat-square
[crates-url]: https://crates.io/crates/nvisy-core
[docs-badge]: https://img.shields.io/docsrs/nvisy-core?logo=Docs.rs&style=flat-square
[docs-url]: http://docs.rs/nvisy-core

Core types and abstractions for the Nvisy content processing system.

This crate provides the fundamental building blocks for content handling, file operations, data classification, and I/O abstractions.

## Features

- **Content Management**: Unified content data structures with SHA256 hashing and metadata
- **File Operations**: Async file handling with content source tracking
- **Data Classification**: Sensitivity levels and structure type classification
- **Format Detection**: Automatic content kind detection from file extensions
- **I/O Abstractions**: Modern async traits for content reading and writing
- **Serialization Support**: Optional serde support for JSON/YAML serialization
- **Zero-Copy Operations**: Efficient data handling using `bytes::Bytes`
