# nvisy.com/error

[![Build Status][action-badge]][action-url]
[![Crate Docs][docs-badge]][docs-url]
[![Crate Version][crates-badge]][crates-url]
[![Crate Coverage][coverage-badge]][coverage-url]

**Check out other `nvisy` projects [here](https://github.com/nvisycom).**

[action-badge]: https://img.shields.io/github/actions/workflow/status/nvisycom/core/build.yaml?branch=main&label=build&logo=github&style=flat-square
[action-url]: https://github.com/nvisycom/core/actions/workflows/build.yaml
[crates-badge]: https://img.shields.io/crates/v/nvisy-error.svg?logo=rust&style=flat-square
[crates-url]: https://crates.io/crates/nvisy-error
[docs-badge]: https://img.shields.io/docsrs/nvisy-error?logo=Docs.rs&style=flat-square
[docs-url]: http://docs.rs/nvisy-error

Structured error handling and component status tracking for the system.

## Core Types

- [`Error`]: Structured errors with source classification and context tracking
- [`ComponentStatus`]: Component health and operational state management

## Features

- `serde` (default): Enables serialization support
- `jiff`: Enables timestamp support for status tracking

## Quick Start

```rust
use nvisy_error::{Error, ErrorType, ErrorResource, ComponentStatus};

// Create typed errors
let error = Error::new(ErrorType::Config, ErrorResource::error, "invalid config")
    .with_context("missing required field");

// Track component status
let status = ComponentStatus::default()
    .with_message("service healthy");
```
