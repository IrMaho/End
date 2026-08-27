use std::fmt;

#[derive(Debug, Clone)]
pub enum RaftError {
    NotLeader { leader_id: Option<u64> },
    QuorumNotReached { votes: usize, required: usize },
    LogMismatch { expected_term: u64, actual_term: u64 },
    StorageError(String),
    NetworkError(String),
    Timeout(String),
    NodeNotFound(u64),
    ClusterStopped,
}

impl fmt::Display for RaftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLeader { leader_id } => write!(f, "Not leader. Current leader: {:?}", leader_id),
            Self::QuorumNotReached { votes, required } => {
                write!(f, "Quorum not reached: {} votes received, {} required", votes, required)
            }
            Self::LogMismatch { expected_term, actual_term } => {
                write!(f, "Log mismatch: expected term {}, found {}", expected_term, actual_term)
            }
            Self::StorageError(msg) => write!(f, "Raft Storage error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Raft Network error: {}", msg),
            Self::Timeout(msg) => write!(f, "Raft Operation timed out: {}", msg),
            Self::NodeNotFound(id) => write!(f, "Node {} not found in cluster configuration", id),
            Self::ClusterStopped => write!(f, "Raft cluster is stopped"),
        }
    }
}

impl std::error::Error for RaftError {}
