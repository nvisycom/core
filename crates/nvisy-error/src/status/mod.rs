//! Component health and operational state tracking with status reporting.

use hipstr::HipStr;
#[cfg(feature = "jiff")]
use jiff::Timestamp;
#[cfg(feature = "jiff")]
use jiff::fmt::serde::timestamp::nanosecond::optional as optional_nanosecond;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use crate::status::health_status::HealthStatus;
pub use crate::status::operational_state::OperationalState;
pub use crate::status::update_severity::UpdateSeverity;
use crate::{Error, ErrorResource, ErrorType, Result};

mod health_status;
mod operational_state;
mod update_severity;

/// Component status tracking health, operational state, and contextual information.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[must_use]
pub struct ComponentStatus {
    /// Current health status of the component.
    pub health_status: HealthStatus,
    /// Current operational state of the component.
    pub operational_state: OperationalState,
    /// Severity level for status updates and alerts.
    pub update_severity: UpdateSeverity,

    /// Descriptive message about the current status.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub message: Option<HipStr<'static>>,
    /// Additional context or diagnostic details.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub context: Option<HipStr<'static>>,

    /// Timestamp when this status was recorded.
    #[cfg(feature = "jiff")]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "serde", serde(with = "optional_nanosecond"))]
    pub timestamp: Option<Timestamp>,
}

impl ComponentStatus {
    /// Creates a new component status.
    pub const fn new(health_status: HealthStatus) -> Self {
        let operational_state = match health_status {
            h if h.is_running() => OperationalState::Running,
            HealthStatus::Offline => OperationalState::Stopped,
            _ => OperationalState::Starting,
        };

        let update_severity = match health_status {
            HealthStatus::Online => UpdateSeverity::Info,
            h if h.is_degraded() => UpdateSeverity::Error,
            _ => UpdateSeverity::Warning,
        };

        Self {
            health_status,
            operational_state,
            update_severity,
            message: None,
            context: None,
            #[cfg(feature = "jiff")]
            timestamp: None,
        }
    }

    /// Sets the health status of the status.
    pub const fn with_health_status(mut self, health_status: HealthStatus) -> Self {
        self.health_status = health_status;
        self
    }

    /// Sets the operational state of the status.
    pub const fn with_operational_state(mut self, operational_state: OperationalState) -> Self {
        self.operational_state = operational_state;
        self
    }

    /// Sets the update severity of the status.
    pub const fn with_update_severity(mut self, update_severity: UpdateSeverity) -> Self {
        self.update_severity = update_severity;
        self
    }

    /// Adds a message to the status.
    pub fn with_message(mut self, message: impl Into<HipStr<'static>>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Adds details to the status.
    pub fn with_details(mut self, context: impl Into<HipStr<'static>>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Adds a timestamp to the status.
    #[cfg(feature = "jiff")]
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Adds the current timestamp to the status.
    #[cfg(feature = "jiff")]
    pub fn with_current_timestamp(mut self) -> Self {
        self.timestamp = Some(Timestamp::now());
        self
    }
}

impl ComponentStatus {
    /// Checks if the component is considered operational.
    #[must_use]
    pub const fn is_operational(&self) -> bool {
        self.operational_state.is_operational() && self.health_status.is_operational()
    }

    /// Checks if the component is considered degraded.
    #[must_use]
    pub const fn is_degraded(&self) -> bool {
        self.health_status.is_degraded()
    }

    /// Checks if the component is in a critical state.
    #[must_use]
    pub const fn is_critical(&self) -> bool {
        self.health_status.is_critical() || self.update_severity.is_critical()
    }

    /// Checks if the component is running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.operational_state.is_running()
    }

    /// Checks if the component is stopped or stopping.
    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        self.operational_state.is_stopped()
    }

    /// Converts the component status into a Result.
    ///
    /// Returns `Ok(())` if the component is operational, otherwise returns an `Err`
    /// with details about the non-operational status using the specified error type.
    pub fn into_result(self, error_type: ErrorType, error_resource: ErrorResource) -> Result<()> {
        if self.is_operational() {
            return Ok(());
        }

        let message = self
            .message
            .unwrap_or_else(|| "Component is not operational".into());
        let mut error = Error::new(error_type, error_resource, message);

        if let Some(context) = self.context {
            error = error.with_context(context);
        }

        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let status = ComponentStatus::new(HealthStatus::MinorDegraded)
            .with_operational_state(OperationalState::Running)
            .with_update_severity(UpdateSeverity::Warning)
            .with_message("test message")
            .with_details("additional details");

        assert_eq!(status.message.as_deref(), Some("test message"));
        assert_eq!(status.context.as_deref(), Some("additional details"));
    }

    #[test]
    fn test_into_result() {
        let status = ComponentStatus::new(HealthStatus::Offline)
            .with_operational_state(OperationalState::Stopped)
            .with_update_severity(UpdateSeverity::Critical)
            .with_message("Component failed")
            .with_details("Database connection lost");

        let result = status.into_result(ErrorType::Other, ErrorResource::Engine);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.error_type, ErrorType::Other);
        assert_eq!(error.error_resource, ErrorResource::Engine);
        assert_eq!(error.message, "Component failed");
        assert_eq!(error.context.as_deref(), Some("Database connection lost"));
    }
}
