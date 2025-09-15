//! Structured error handling with source classification and context tracking.

use hipstr::HipStr;

pub use crate::error::error_source::ErrorResource;
pub use crate::error::error_type::ErrorType;

mod error_source;
mod error_type;

/// Type alias for boxed standard errors.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Structured error type with source classification and context tracking.
#[must_use]
#[derive(Debug, thiserror::Error)]
#[error("{}", self.display_message())]
pub struct Error {
    /// Error classification type.
    pub error_type: ErrorType,
    /// Component where the error originated.
    pub error_resource: ErrorResource,

    /// Underlying source error, if any.
    #[source]
    pub source: Option<BoxError>,
    /// Additional context information.
    pub context: Option<HipStr<'static>>,
    /// Primary error message.
    pub message: HipStr<'static>,
}

/// Result type alias using the nvisy Error.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Creates a new error with the specified type, source, and message.
    pub fn new(
        error_type: ErrorType,
        error_resource: ErrorResource,
        message: impl Into<HipStr<'static>>,
    ) -> Self {
        Self {
            error_type,
            error_resource,
            source: None,
            context: None,
            message: message.into(),
        }
    }

    /// Creates a new error with the specified type, source, message, and source error.
    pub fn from_source(
        error_type: ErrorType,
        error_source: ErrorResource,
        message: impl Into<HipStr<'static>>,
        source: impl Into<BoxError>,
    ) -> Self {
        Self {
            error_type,
            error_resource: error_source,
            source: Some(source.into()),
            context: None,
            message: message.into(),
        }
    }

    /// Set the type of the error.
    pub const fn with_type(mut self, error_type: ErrorType) -> Self {
        self.error_type = error_type;
        self
    }

    /// Set the source of the error.
    pub const fn with_source(mut self, error_resource: ErrorResource) -> Self {
        self.error_resource = error_resource;
        self
    }

    /// Adds context to the error.
    pub fn with_context(mut self, context: impl Into<HipStr<'static>>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Returns the display message for the error.
    fn display_message(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!(
            "[{}:{}]",
            self.error_resource.as_ref(),
            self.error_type.as_ref()
        ));
        parts.push(self.message.to_string());

        if let Some(ref context) = self.context {
            parts.push(format!("(context: {context})"));
        }

        parts.join(" ")
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::from_source(
            ErrorType::Runtime,
            ErrorResource::Core,
            "I/O operation failed",
            error,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_builder() {
        let error = Error::new(ErrorType::Config, ErrorResource::Core, "test message");
        assert_eq!(error.error_type, ErrorType::Config);
        assert_eq!(error.error_resource, ErrorResource::Core);
        assert_eq!(error.message, "test message");
        assert!(error.source.is_none());
        assert!(error.context.is_none());
    }

    #[test]
    fn test_error_with_context() {
        let error = Error::new(ErrorType::Other, ErrorResource::Engine, "test")
            .with_context("additional context");
        assert_eq!(error.context.as_deref(), Some("additional context"));
    }
}
