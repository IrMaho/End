use super::error::RaftError;
use super::types::RaftMessage;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone, Default)]
pub struct PartitionController {
    isolated_pairs: Arc<Mutex<HashSet<(u64, u64)>>>,
}

impl PartitionController {
    pub fn new() -> Self {
        Self {
            isolated_pairs: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Isolate `node_id` from all other nodes in the partition list
    pub fn partition_node(&self, node_id: u64, other_nodes: &[u64]) {
        let mut set = self.isolated_pairs.lock().unwrap();
        for &other in other_nodes {
            if other != node_id {
                set.insert((node_id, other));
                set.insert((other, node_id));
            }
        }
    }

    /// Isolate a group of nodes (minority) from another group of nodes (majority)
    pub fn create_partition(&self, group_a: &[u64], group_b: &[u64]) {
        let mut set = self.isolated_pairs.lock().unwrap();
        for &a in group_a {
            for &b in group_b {
                set.insert((a, b));
                set.insert((b, a));
            }
        }
    }

    /// Heal all partitions across the cluster
    pub fn heal_all(&self) {
        let mut set = self.isolated_pairs.lock().unwrap();
        set.clear();
    }

    /// Heal partition between specific nodes
    pub fn heal_node(&self, node_id: u64) {
        let mut set = self.isolated_pairs.lock().unwrap();
        set.retain(|(a, b)| *a != node_id && *b != node_id);
    }

    /// Check if connection between `from` and `to` is blocked by a network partition
    pub fn is_blocked(&self, from: u64, to: u64) -> bool {
        let set = self.isolated_pairs.lock().unwrap();
        set.contains(&(from, to))
    }
}

pub struct TcpRaftTransport {
    pub node_id: u64,
    pub partition_controller: PartitionController,
}

impl TcpRaftTransport {
    pub fn new(node_id: u64, partition_controller: PartitionController) -> Self {
        Self {
            node_id,
            partition_controller,
        }
    }

    pub async fn send_rpc(
        &self,
        target_node_id: u64,
        target_addr: &str,
        msg: RaftMessage,
    ) -> Result<RaftMessage, RaftError> {
        // Enforce network partition rules
        if self.partition_controller.is_blocked(self.node_id, target_node_id) {
            return Err(RaftError::NetworkError(format!(
                "Network partition: node {} is isolated from peer {}",
                self.node_id, target_node_id
            )));
        }

        // Establish TCP connection with 500ms timeout
        let connect_fut = TcpStream::connect(target_addr);
        let mut stream = tokio::time::timeout(Duration::from_millis(500), connect_fut)
            .await
            .map_err(|_| RaftError::Timeout(format!("Connection to {} timed out", target_addr)))?
            .map_err(|e| RaftError::NetworkError(format!("Failed to connect to {}: {}", target_addr, e)))?;

        // Double check partition before sending in case state changed
        if self.partition_controller.is_blocked(self.node_id, target_node_id) {
            return Err(RaftError::NetworkError(format!(
                "Network partition: node {} is isolated from peer {}",
                self.node_id, target_node_id
            )));
        }

        // Serialize message
        let payload = serde_json::to_vec(&msg)
            .map_err(|e| RaftError::NetworkError(format!("Serialization error: {}", e)))?;
        let len_bytes = (payload.len() as u32).to_be_bytes();

        // Write length + body
        stream
            .write_all(&len_bytes)
            .await
            .map_err(|e| RaftError::NetworkError(format!("Failed to write length: {}", e)))?;
        stream
            .write_all(&payload)
            .await
            .map_err(|e| RaftError::NetworkError(format!("Failed to write payload: {}", e)))?;
        stream
            .flush()
            .await
            .map_err(|e| RaftError::NetworkError(format!("Failed to flush stream: {}", e)))?;

        // Read response with 800ms timeout
        let read_fut = async {
            let mut resp_len_buf = [0u8; 4];
            stream.read_exact(&mut resp_len_buf).await?;
            let resp_len = u32::from_be_bytes(resp_len_buf) as usize;
            let mut resp_buf = vec![0u8; resp_len];
            stream.read_exact(&mut resp_buf).await?;
            Ok::<Vec<u8>, std::io::Error>(resp_buf)
        };

        let resp_bytes = tokio::time::timeout(Duration::from_millis(800), read_fut)
            .await
            .map_err(|_| RaftError::Timeout(format!("Response from {} timed out", target_addr)))?
            .map_err(|e| RaftError::NetworkError(format!("Failed to read response from {}: {}", target_addr, e)))?;

        let response: RaftMessage = serde_json::from_slice(&resp_bytes)
            .map_err(|e| RaftError::NetworkError(format!("Failed to deserialize response: {}", e)))?;

        Ok(response)
    }
}
