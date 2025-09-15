# nvisy.com

[![Build Status][action-badge]][action-url]
[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]

[action-badge]: https://img.shields.io/github/actions/workflow/status/nvisycom/core/build.yaml
[action-url]: https://github.com/nvisycom/core/actions/workflows/build.yaml
[crates-badge]: https://img.shields.io/crates/v/nvisy-core
[crates-url]: https://crates.io/crates/nvisy-core
[docs-badge]: https://docs.rs/nvisy-core/badge.svg
[docs-url]: https://docs.rs/nvisy-core

A comprehensive, high-performance data redaction library for Rust that
automatically detects and redacts sensitive information from text, structured
data, and files.

## 🚀 Features

- **Multi-format Support**: Text, JSON, XML, YAML, TOML, CSV, logs, and more
- **Advanced Pattern Detection**: Regex-based and ML-powered sensitive data
  detection
- **35+ Built-in Data Types**: Email addresses, credit cards, SSNs, API keys,
  and more
- **Flexible Replacement Strategies**: Masking, tokenization, hashing, and
  custom replacements
- **High Performance**: Parallel processing with memory-efficient streaming
- **Async/Await Support**: Full tokio integration for non-blocking operations
- **Memory Safe**: Zero unsafe code with secure handling of sensitive data
- **Configurable Policies**: Fine-grained control over redaction rules
- **Progress Tracking**: Built-in progress reporting for long-running operations
- **Comprehensive Reporting**: Detailed analytics and audit trails

---

let's work on nvisy_core

- fix docs in lib.rs by adding in doc links to mentioned structs i.e. [ContentMetadata]: fs::ContentMetadata at the botton of the //! comment, etc
- io,fs,path modules shouldn't make their internal modules public, but they should still reexport all content of their internal modules
- rename DataStructureType into DataStructureKind
- add method to SupportedFormat that returns DataStructureKind
- delete FormatType, replace all of its uses with ContentKind
- ContentKind::from_file_extension should return ContentKind::Unknown if the extension is not recognized
- add default impl for ContentKind (ContentKind::Unknown)
- ContentFile should have its fields private and provide getters for path/source, and add as_file and into_file methods
- rename DataSensitivityLevel into DataSensitivity
- comment types in SupportedFormat
- add mutex around sha256 in ContentData so that we don't need it mutable when getting the hash
- fix all errors in nvisy_core, apply best practices

---

let's work on nvisy_archive

-
