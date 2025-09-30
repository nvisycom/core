#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

/// Severity level for status updates indicating the urgency and importance of alerts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum UpdateSeverity {
    /// Informational updates requiring no immediate action.
    #[default]
    Info,
    /// Warning conditions that may require attention.
    Warning,
    /// Error conditions requiring prompt investigation.
    Error,
    /// Critical conditions requiring immediate response.
    Critical,
}

impl UpdateSeverity {
    /// Returns `true` if the severity requires immediate attention.
    #[must_use]
    pub const fn is_critical(&self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Returns `true` if the severity indicates an error condition or worse.
    #[must_use]
    pub const fn is_error_or_higher(&self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }

    /// Returns `true` if the severity indicates a warning condition or worse.
    #[must_use]
    pub const fn is_warning_or_higher(&self) -> bool {
        matches!(self, Self::Warning | Self::Error | Self::Critical)
    }

    /// Returns the numeric priority level for sorting and comparison.
    ///
    /// Higher values indicate higher severity.
    #[must_use]
    pub const fn priority_level(&self) -> u8 {
        match self {
            Self::Info => 0,
            Self::Warning => 1,
            Self::Error => 2,
            Self::Critical => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_representations() {
        assert_eq!(UpdateSeverity::Info.as_ref(), "info");
        assert_eq!(UpdateSeverity::Warning.as_ref(), "warning");
        assert_eq!(UpdateSeverity::Error.as_ref(), "error");
        assert_eq!(UpdateSeverity::Critical.as_ref(), "critical");
    }

    #[test]
    fn test_severity_levels() {
        assert!(UpdateSeverity::Critical.is_critical());
        assert!(!UpdateSeverity::Error.is_critical());

        assert!(UpdateSeverity::Error.is_error_or_higher());
        assert!(UpdateSeverity::Critical.is_error_or_higher());
        assert!(!UpdateSeverity::Warning.is_error_or_higher());

        assert!(UpdateSeverity::Warning.is_warning_or_higher());
        assert!(UpdateSeverity::Error.is_warning_or_higher());
        assert!(UpdateSeverity::Critical.is_warning_or_higher());
        assert!(!UpdateSeverity::Info.is_warning_or_higher());
    }

    #[test]
    fn test_priority_levels() {
        assert_eq!(UpdateSeverity::Info.priority_level(), 0);
        assert_eq!(UpdateSeverity::Warning.priority_level(), 1);
        assert_eq!(UpdateSeverity::Error.priority_level(), 2);
        assert_eq!(UpdateSeverity::Critical.priority_level(), 3);

        // Test ordering
        assert!(UpdateSeverity::Critical.priority_level() > UpdateSeverity::Error.priority_level());
        assert!(UpdateSeverity::Error.priority_level() > UpdateSeverity::Warning.priority_level());
        assert!(UpdateSeverity::Warning.priority_level() > UpdateSeverity::Info.priority_level());
    }
}
