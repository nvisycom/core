#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub mod error;
pub mod status;

// Re-export main types for convenience

pub use error::{BoxError, Error, ErrorResource, ErrorType, Result};
pub use status::{ComponentStatus, HealthStatus, OperationalState, UpdateSeverity};

/// Trait for components that can report their operational status and health.
///
/// This trait defines a standardized interface for system components to provide
/// both real-time and cached status information asynchronously. Components that
/// implement this trait can be monitored for health, operational state, and
/// performance characteristics.
///
/// # Usage
///
/// Components should implement this trait to enable system-wide monitoring
/// and health checks. The trait provides two methods for status reporting:
/// - [`current_status`] for real-time status checks (potentially expensive)
/// - [`cached_status`] for quick status retrieval from cache (if available)
///
/// # Error Handling
///
/// Status information can be converted to a [`Result`] using the
/// [`ComponentStatus::into_result`] method, which allows for easy
/// integration with error handling patterns:
///
/// [`current_status`]: Component::current_status
/// [`cached_status`]: Component::cached_status
pub trait Component: std::fmt::Debug {
    /// Returns the current operational status of the component.
    ///
    /// This method performs real-time health and operational checks to determine
    /// the component's current state. Implementations should include appropriate
    /// checks for connectivity, resource availability, and functionality.
    ///
    /// # Performance Considerations
    ///
    /// This method may perform expensive operations such as network calls,
    /// database queries, or file system checks. For frequent status polling,
    /// consider using [`cached_status`] when available.
    ///
    /// [`cached_status`]: Component::cached_status
    fn current_status(&self) -> impl Future<Output = ComponentStatus>;

    /// Returns a cached status if available, otherwise returns `None`.
    ///
    /// This method provides access to previously computed status information
    /// without performing expensive real-time checks. Components may implement
    /// caching strategies to improve performance for frequent status queries.
    ///
    /// # Return Value
    ///
    /// - `Some(ComponentStatus)` if cached status information is available
    /// - `None` if no cached status exists or caching is not implemented
    fn cached_status(&self) -> impl Future<Output = Option<ComponentStatus>>;
}

#[doc(hidden)]
pub mod prelude {
    //! Prelude module for commonly used types.
    //!
    //! This module re-exports the most commonly used types from this crate.
    //! It is intended to be glob-imported for convenience.

    pub use crate::Component;
    pub use crate::error::{Error, ErrorResource, ErrorType, Result, BoxError};
    pub use crate::status::{ComponentStatus, HealthStatus, OperationalState, UpdateSeverity};
}
