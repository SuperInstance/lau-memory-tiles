use serde::{Deserialize, Serialize};
use std::fmt;

/// The type of a memory tile — what kind of cognitive event it records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileType {
    /// What happened — a perception or event.
    Observation,
    /// What was done — an action taken.
    Action,
    /// Internal reasoning — a thought.
    Thought,
    /// A choice made, with provenance.
    Decision,
    /// Consolidation produced during idle / dream phase.
    Dream,
    /// Something went wrong.
    Error,
    /// Data from a sensor or port.
    Signal,
    /// Output sent to a user or agent.
    Response,
    /// Custom tile type with a free-form label.
    Custom(String),
}

impl fmt::Display for TileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TileType::Observation => write!(f, "observation"),
            TileType::Action => write!(f, "action"),
            TileType::Thought => write!(f, "thought"),
            TileType::Decision => write!(f, "decision"),
            TileType::Dream => write!(f, "dream"),
            TileType::Error => write!(f, "error"),
            TileType::Signal => write!(f, "signal"),
            TileType::Response => write!(f, "response"),
            TileType::Custom(s) => write!(f, "custom:{s}"),
        }
    }
}

impl TileType {
    /// Returns true if this tile type represents an externally observable event.
    pub fn is_observable(&self) -> bool {
        matches!(self, TileType::Observation | TileType::Action | TileType::Signal | TileType::Response)
    }

    /// Returns true if this tile type represents an internal cognitive event.
    pub fn is_internal(&self) -> bool {
        matches!(self, TileType::Thought | TileType::Decision | TileType::Dream)
    }
}
