#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

/// Classification of error types by their operational domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ErrorType {
    /// Configuration loading, parsing, or validation failures.
    Config,
    /// Execution-time operational failures.
    Runtime,
    /// Internal system logic or state failures.
    Other,
}

impl ErrorType {
    /// Check if this error type is typically recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self, ErrorType::Runtime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recoverability() {
        assert!(ErrorType::Runtime.is_recoverable());
        assert!(!ErrorType::Other.is_recoverable());
        assert!(!ErrorType::Config.is_recoverable());
    }
}
