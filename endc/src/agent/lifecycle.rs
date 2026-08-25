use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Explicit lifecycle states for an Agent Contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleState {
    #[serde(rename = "DRAFT")]
    Draft,
    #[serde(rename = "SUBMITTED")]
    Submitted,
    #[serde(rename = "VERIFYING")]
    Verifying,
    #[serde(rename = "VERIFIED")]
    Verified,
    #[serde(rename = "REJECTED")]
    Rejected,
    #[serde(rename = "STALE")]
    Stale,
}

impl LifecycleState {
    /// Check if a transition from `self` to `target` is mechanically valid.
    pub fn can_transition_to(self, target: LifecycleState) -> bool {
        matches!(
            (self, target),
            (LifecycleState::Draft, LifecycleState::Submitted)
                | (LifecycleState::Submitted, LifecycleState::Verifying)
                | (LifecycleState::Verifying, LifecycleState::Verified)
                | (LifecycleState::Verifying, LifecycleState::Rejected)
                | (LifecycleState::Verified, LifecycleState::Stale)
                | (LifecycleState::Stale, LifecycleState::Submitted)
                | (LifecycleState::Rejected, LifecycleState::Submitted)
        )
    }

    /// Perform a validated state transition, returning a structured error if illegal.
    pub fn transition(&mut self, target: LifecycleState) -> Result<(), LifecycleError> {
        if self.can_transition_to(target) {
            *self = target;
            Ok(())
        } else {
            Err(LifecycleError {
                current: *self,
                target,
                reason: format!(
                    "Illegal lifecycle transition from {:?} to {:?}. Allowed transitions: {}",
                    *self,
                    target,
                    self.allowed_transitions_str()
                ),
            })
        }
    }

    /// Helper describing valid next states from the current state.
    pub fn allowed_transitions_str(self) -> &'static str {
        match self {
            LifecycleState::Draft => "SUBMITTED",
            LifecycleState::Submitted => "VERIFYING",
            LifecycleState::Verifying => "VERIFIED, REJECTED",
            LifecycleState::Verified => "STALE",
            LifecycleState::Rejected => "SUBMITTED",
            LifecycleState::Stale => "SUBMITTED",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LifecycleState::Draft => "DRAFT",
            LifecycleState::Submitted => "SUBMITTED",
            LifecycleState::Verifying => "VERIFYING",
            LifecycleState::Verified => "VERIFIED",
            LifecycleState::Rejected => "REJECTED",
            LifecycleState::Stale => "STALE",
        }
    }
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LifecycleState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "DRAFT" => Ok(LifecycleState::Draft),
            "SUBMITTED" => Ok(LifecycleState::Submitted),
            "VERIFYING" => Ok(LifecycleState::Verifying),
            "VERIFIED" => Ok(LifecycleState::Verified),
            "REJECTED" => Ok(LifecycleState::Rejected),
            "STALE" => Ok(LifecycleState::Stale),
            other => Err(format!(
                "Unknown lifecycle state: '{}'. Must be one of: DRAFT, SUBMITTED, VERIFYING, VERIFIED, REJECTED, STALE",
                other
            )),
        }
    }
}

/// Structured error describing an invalid lifecycle transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleError {
    pub current: LifecycleState,
    pub target: LifecycleState,
    pub reason: String,
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lifecycle Transition Error: cannot transition from {} to {} ({})",
            self.current, self.target, self.reason
        )
    }
}

impl std::error::Error for LifecycleError {}
