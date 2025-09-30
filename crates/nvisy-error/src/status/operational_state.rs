#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

/// Component operational state indicating current execution phase and lifecycle.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum OperationalState {
    /// Component is initializing and preparing to run.
    Starting,
    /// Component is fully operational and processing requests.
    #[default]
    Running,
    /// Component is gracefully shutting down.
    Stopping,
    /// Component has completed shutdown and is not operational.
    Stopped,
}

impl OperationalState {
    /// Returns `true` if the component can process requests or perform work.
    #[must_use]
    pub const fn is_operational(&self) -> bool {
        matches!(self, Self::Starting | Self::Running)
    }

    /// Returns `true` if the component is fully operational and processing requests.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns `true` if the component is shutdown or in the process of shutting down.
    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        matches!(self, Self::Stopping | Self::Stopped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_representations() {
        assert_eq!(OperationalState::Starting.as_ref(), "starting");
        assert_eq!(OperationalState::Running.as_ref(), "running");
        assert_eq!(OperationalState::Stopping.as_ref(), "stopping");
        assert_eq!(OperationalState::Stopped.as_ref(), "stopped");
    }
}
