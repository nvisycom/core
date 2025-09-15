# nvisy.com/archive

[![Build Status][action-badge]][action-url]
[![Crate Docs][docs-badge]][docs-url]
[![Crate Version][crates-badge]][crates-url]
[![Crate Coverage][coverage-badge]][coverage-url]

**Check out other `nvisy` projects [here](https://github.com/nvisycom).**

[action-badge]: https://img.shields.io/github/actions/workflow/status/nvisycom/core/build.yaml?branch=main&label=build&logo=github&style=flat-square
[action-url]: https://github.com/nvisycom/core/actions/workflows/build.yaml
[crates-badge]: https://img.shields.io/crates/v/nvisy-archive.svg?logo=rust&style=flat-square
[crates-url]: https://crates.io/crates/nvisy-archive
[docs-badge]: https://img.shields.io/docsrs/nvisy-archive?logo=Docs.rs&style=flat-square
[docs-url]: http://docs.rs/nvisy-archive

A comprehensive archive handling library for the nvisy ecosystem. This crate provides functionality for working with various archive formats including ZIP, TAR, and compressed variants.

## Features

- **Multiple Archive Formats**: Support for ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ, GZIP, BZIP2, and XZ
- **Flexible Loading**: Load archives from file paths, memory, or byte iterators
- **Async Operations**: Full async/await support with tokio
- **Type Safety**: Strong typing with `ArchiveType` enum for format detection
- **Memory Efficient**: Stream-based processing for large archives
- **Cross-platform**: Works on Windows, macOS, and Linux
