#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

/// Component health status indicating operational wellness and degradation levels.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum HealthStatus {
    /// Component is fully operational and healthy.
    #[default]
    Online,
    /// Component is operational but experiencing minor issues.
    MinorDegraded,
    /// Component is experiencing significant issues but still functional.
    MajorDegraded,
    /// Component has failed and is not operational.
    Offline,
    /// Component status cannot be determined.
    Unknown,
}

impl HealthStatus {
    /// Returns `true` if the component is in a critical state requiring immediate attention.
    #[must_use]
    pub const fn is_critical(&self) -> bool {
        matches!(self, Self::Offline)
    }

    /// Returns `true` if the component is running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(
            self,
            Self::Online | Self::MinorDegraded | Self::MajorDegraded
        )
    }

    /// Returns `true` if the component can perform its primary functions.
    #[must_use]
    pub const fn is_operational(&self) -> bool {
        matches!(self, Self::Online | Self::MinorDegraded)
    }

    /// Returns `true` if the component is experiencing any level of degradation.
    #[must_use]
    pub const fn is_degraded(&self) -> bool {
        matches!(
            self,
            Self::MinorDegraded | Self::MajorDegraded | Self::Offline
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_representations() {
        assert_eq!(HealthStatus::Online.as_ref(), "online");
        assert_eq!(HealthStatus::MinorDegraded.as_ref(), "minor_degraded");
        assert_eq!(HealthStatus::MajorDegraded.as_ref(), "major_degraded");
        assert_eq!(HealthStatus::Offline.as_ref(), "offline");
        assert_eq!(HealthStatus::Unknown.as_ref(), "unknown");
    }
}
