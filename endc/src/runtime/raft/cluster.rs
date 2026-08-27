use super::error::RaftError;
use super::node::RaftNodeServer;
use super::transport::PartitionController;
use super::types::{ClusterStatusReply, RaftRole};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct RaftCluster {
    pub runtime: Option<Arc<tokio::runtime::Runtime>>,
    pub node_count: usize,
    pub base_port: u16,
    pub base_db_path: String,
    pub partition_controller: PartitionController,
    pub nodes: HashMap<u64, Arc<RaftNodeServer>>,
    pub node_addresses: HashMap<u64, String>,
}

impl RaftCluster {
    pub async fn start(node_count: usize, base_port: u16, base_db_path: &str) -> Result<Self, RaftError> {
        Self::start_internal(None, node_count, base_port, base_db_path).await
    }

    pub fn start_sync(node_count: usize, base_port: u16, base_db_path: &str) -> Result<Self, RaftError> {
        let rt = Arc::new(tokio::runtime::Runtime::new().map_err(|e| RaftError::NetworkError(e.to_string()))?);
        let _guard = rt.enter();
        rt.block_on(Self::start_internal(Some(rt.clone()), node_count, base_port, base_db_path))
    }

    async fn start_internal(
        runtime: Option<Arc<tokio::runtime::Runtime>>,
        node_count: usize,
        base_port: u16,
        base_db_path: &str,
    ) -> Result<Self, RaftError> {
        let partition_controller = PartitionController::new();
        let mut node_addresses = HashMap::new();

        for i in 1..=(node_count as u64) {
            let addr = format!("127.0.0.1:{}", base_port + (i as u16));
            node_addresses.insert(i, addr);
        }

        let mut nodes = HashMap::new();

        for i in 1..=(node_count as u64) {
            let mut peers = HashMap::new();
            for (&peer_id, peer_addr) in &node_addresses {
                if peer_id != i {
                    peers.insert(peer_id, peer_addr.clone());
                }
            }

            let db_file = if base_db_path.is_empty() || base_db_path == ":memory:" {
                ":memory:".to_string()
            } else {
                format!("{}_node_{}.db", base_db_path, i)
            };

            let server = Arc::new(RaftNodeServer::new(
                i,
                node_addresses[&i].clone(),
                peers,
                db_file,
                partition_controller.clone(),
            )?);
            server.start().await?;
            nodes.insert(i, server);
        }

        Ok(Self {
            runtime,
            node_count,
            base_port,
            base_db_path: base_db_path.to_string(),
            partition_controller,
            nodes,
            node_addresses,
        })
    }

    pub fn get_leader_id(&self) -> Option<u64> {
        for server in self.nodes.values() {
            let status = server.get_status();
            if status.is_active && status.role == RaftRole::Leader {
                return Some(status.node_id);
            }
        }
        None
    }

    pub async fn wait_for_leader(&self, timeout: Duration) -> Result<u64, RaftError> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if let Some(leader_id) = self.get_leader_id() {
                return Ok(leader_id);
            }
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
        Err(RaftError::Timeout(format!(
            "Leader election timed out after {} ms",
            timeout.as_millis()
        )))
    }

    pub fn wait_for_leader_sync(&self, timeout: Duration) -> Result<u64, RaftError> {
        if let Some(rt) = &self.runtime {
            let _guard = rt.enter();
            rt.block_on(self.wait_for_leader(timeout))
        } else {
            let start = Instant::now();
            while start.elapsed() < timeout {
                if let Some(leader_id) = self.get_leader_id() {
                    return Ok(leader_id);
                }
                std::thread::sleep(Duration::from_millis(30));
            }
            Err(RaftError::Timeout(format!("Leader election timed out after {} ms", timeout.as_millis())))
        }
    }

    pub async fn write(&self, command: &str, payload: &str) -> Result<u64, RaftError> {
        let leader_id = self
            .wait_for_leader(Duration::from_secs(3))
            .await?;

        let leader_server = self
            .nodes
            .get(&leader_id)
            .ok_or(RaftError::NodeNotFound(leader_id))?;

        leader_server
            .write(command.to_string(), payload.to_string(), Duration::from_millis(1500))
            .await
    }

    pub fn write_sync(&self, command: &str, payload: &str) -> Result<u64, RaftError> {
        if let Some(rt) = &self.runtime {
            let _guard = rt.enter();
            rt.block_on(self.write(command, payload))
        } else {
            Err(RaftError::NetworkError("No runtime context".to_string()))
        }
    }

    pub fn read(&self, key: &str) -> Result<Option<String>, RaftError> {
        if let Some(leader_id) = self.get_leader_id() {
            if let Some(server) = self.nodes.get(&leader_id) {
                return server.read(key);
            }
        }
        for server in self.nodes.values() {
            if server.get_status().is_active {
                if let Ok(res) = server.read(key) {
                    if res.is_some() {
                        return Ok(res);
                    }
                }
            }
        }
        Ok(None)
    }

    pub fn read_from_node(&self, node_id: u64, key: &str) -> Result<Option<String>, RaftError> {
        let server = self
            .nodes
            .get(&node_id)
            .ok_or(RaftError::NodeNotFound(node_id))?;
        server.read(key)
    }

    pub fn get_node_status(&self, node_id: u64) -> Result<ClusterStatusReply, RaftError> {
        let server = self
            .nodes
            .get(&node_id)
            .ok_or(RaftError::NodeNotFound(node_id))?;
        Ok(server.get_status())
    }

    pub fn kill_node(&mut self, node_id: u64) -> Result<(), RaftError> {
        if let Some(server) = self.nodes.get(&node_id) {
            server.stop();
            Ok(())
        } else {
            Err(RaftError::NodeNotFound(node_id))
        }
    }

    pub async fn restart_node(&mut self, node_id: u64) -> Result<(), RaftError> {
        let mut peers = HashMap::new();
        for (&peer_id, peer_addr) in &self.node_addresses {
            if peer_id != node_id {
                peers.insert(peer_id, peer_addr.clone());
            }
        }

        let db_file = if self.base_db_path.is_empty() || self.base_db_path == ":memory:" {
            ":memory:".to_string()
        } else {
            format!("{}_node_{}.db", self.base_db_path, node_id)
        };

        let server = Arc::new(RaftNodeServer::new(
            node_id,
            self.node_addresses[&node_id].clone(),
            peers,
            db_file,
            self.partition_controller.clone(),
        )?);
        server.start().await?;
        self.nodes.insert(node_id, server);
        Ok(())
    }

    pub fn restart_node_sync(&mut self, node_id: u64) -> Result<(), RaftError> {
        let rt_opt = self.runtime.clone();
        if let Some(rt) = rt_opt {
            let _guard = rt.enter();
            rt.block_on(self.restart_node(node_id))
        } else {
            Err(RaftError::NetworkError("No runtime context".to_string()))
        }
    }

    pub fn partition_node(&self, node_id: u64) {
        let all_ids: Vec<u64> = self.node_addresses.keys().copied().collect();
        self.partition_controller.partition_node(node_id, &all_ids);
    }

    pub fn create_partition(&self, group_a: &[u64], group_b: &[u64]) {
        self.partition_controller.create_partition(group_a, group_b);
    }

    pub fn heal_partition(&self) {
        self.partition_controller.heal_all();
    }

    pub fn stop_all(&self) {
        for server in self.nodes.values() {
            server.stop();
        }
    }
}
