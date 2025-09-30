#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

/// System component sources where errors can originate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ErrorResource {
    /// Core framework and foundational components.
    Core,
    /// Execution engine and processing components.
    Engine,
    /// Pattern matching and rule processing components.
    Pattern,
    /// Runtime environment and dynamic execution components.
    Runtime,
    /// Gateway and API boundary components.
    Gateway,
}

impl ErrorResource {
    /// Returns `true` if the error source is from internal system components.
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        matches!(self, Self::Core | Self::Pattern | Self::Engine)
    }

    /// Returns `true` if the error source is from external or runtime components.
    #[must_use]
    pub const fn is_external(&self) -> bool {
        matches!(self, Self::Runtime | Self::Gateway )
    }

    /// Returns the priority level of the error source for logging and alerting.
    ///
    /// Higher values indicate more critical components.
    #[must_use]
    pub const fn priority_level(&self) -> u8 {
        match self {
            Self::Core => 6, // Highest priority
            Self::Engine => 5,
            Self::Pattern => 4,
            Self::Runtime => 3,
            Self::Gateway => 2, // Lowest priority
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_representations() {
        assert_eq!(ErrorResource::Core.as_ref(), "core");
        assert_eq!(ErrorResource::Engine.as_ref(), "engine");
        assert_eq!(ErrorResource::Pattern.as_ref(), "pattern");
        assert_eq!(ErrorResource::Runtime.as_ref(), "runtime");
        assert_eq!(ErrorResource::Gateway.as_ref(), "gateway");
    }

    #[test]
    fn test_priority_levels() {
        assert_eq!(ErrorResource::Core.priority_level(), 6);
        assert_eq!(ErrorResource::Engine.priority_level(), 5);
        assert_eq!(ErrorResource::Pattern.priority_level(), 4);
        assert_eq!(ErrorResource::Runtime.priority_level(), 3);
        assert_eq!(ErrorResource::Gateway.priority_level(), 2);
    }

    #[test]
    fn test_internal_external_classification() {
        assert!(ErrorResource::Core.is_internal());
        assert!(ErrorResource::Pattern.is_internal());
        assert!(ErrorResource::Engine.is_internal());
        assert!(ErrorResource::Runtime.is_external());
        assert!(ErrorResource::Gateway.is_external());
    }
}
