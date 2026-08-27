use super::error::RaftError;
use super::state::RaftCore;
use super::storage::SqliteRaftStorage;
use super::transport::{PartitionController, TcpRaftTransport};
use super::types::{
    AppendEntriesArgs, AppendEntriesReply, ClientReadArgs, ClientReadReply, ClientWriteArgs, ClientWriteReply,
    ClusterStatusReply, RaftMessage, RaftRole, RequestVoteArgs, RequestVoteReply,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Notify;

pub struct RaftNodeServer {
    pub node_id: u64,
    pub listen_addr: String,
    pub core: Arc<Mutex<RaftCore>>,
    pub transport: Arc<TcpRaftTransport>,
    pub is_running: Arc<AtomicBool>,
    pub notify_commit: Arc<Notify>,
    pub partition_controller: PartitionController,
    pub stop_tx: tokio::sync::watch::Sender<bool>,
    pub stop_rx: tokio::sync::watch::Receiver<bool>,
}

impl RaftNodeServer {
    pub fn new(
        node_id: u64,
        listen_addr: String,
        peers: HashMap<u64, String>,
        db_path: String,
        partition_controller: PartitionController,
    ) -> Result<Self, RaftError> {
        let storage = SqliteRaftStorage::open(&db_path)?;
        let core = Arc::new(Mutex::new(RaftCore::new(node_id, peers, storage)?));
        let transport = Arc::new(TcpRaftTransport::new(node_id, partition_controller.clone()));
        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);

        Ok(Self {
            node_id,
            listen_addr,
            core,
            transport,
            is_running: Arc::new(AtomicBool::new(false)),
            notify_commit: Arc::new(Notify::new()),
            partition_controller,
            stop_tx,
            stop_rx,
        })
    }

    pub async fn start(&self) -> Result<(), RaftError> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.is_running.store(true, Ordering::SeqCst);

        // 1. Start RPC TCP Listener
        let listener = TcpListener::bind(&self.listen_addr)
            .await
            .map_err(|e| RaftError::NetworkError(format!("Failed to bind {}: {}", self.listen_addr, e)))?;

        let is_running_clone = self.is_running.clone();
        let core_clone = self.core.clone();
        let notify_clone = self.notify_commit.clone();
        let node_id = self.node_id;
        let mut stop_rx = self.stop_rx.clone();

        tokio::spawn(async move {
            while is_running_clone.load(Ordering::SeqCst) {
                tokio::select! {
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((mut socket, _peer_addr)) => {
                                let core = core_clone.clone();
                                let notify = notify_clone.clone();
                                tokio::spawn(async move {
                                    let mut len_buf = [0u8; 4];
                                    if socket.read_exact(&mut len_buf).await.is_err() {
                                        return;
                                    }
                                    let len = u32::from_be_bytes(len_buf) as usize;
                                    let mut payload = vec![0u8; len];
                                    if socket.read_exact(&mut payload).await.is_err() {
                                        return;
                                    }

                                    let msg: Result<RaftMessage, _> = serde_json::from_slice(&payload);
                                    if let Ok(request) = msg {
                                        let response = Self::dispatch_rpc(core, notify, node_id, request);
                                        if let Ok(resp_bytes) = serde_json::to_vec(&response) {
                                            let resp_len = (resp_bytes.len() as u32).to_be_bytes();
                                            let _ = socket.write_all(&resp_len).await;
                                            let _ = socket.write_all(&resp_bytes).await;
                                            let _ = socket.flush().await;
                                        }
                                    }
                                });
                            }
                            Err(_) => {
                                if !is_running_clone.load(Ordering::SeqCst) {
                                    break;
                                }
                            }
                        }
                    }
                    _ = stop_rx.changed() => {
                        break;
                    }
                }
            }
        });

        // 2. Start Election Ticker Task
        let core_ticker = self.core.clone();
        let transport_ticker = self.transport.clone();
        let running_ticker = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(25));
            while running_ticker.load(Ordering::SeqCst) {
                interval.tick().await;

                let should_elect = {
                    let core = core_ticker.lock().unwrap();
                    if core.role == RaftRole::Leader {
                        false
                    } else {
                        // Randomized election timeout (150ms - 280ms)
                        let timeout_ms = 150 + ((core.node_id * 37) % 130);
                        core.last_heartbeat_instant.elapsed() > Duration::from_millis(timeout_ms)
                    }
                };

                if should_elect && running_ticker.load(Ordering::SeqCst) {
                    let election_data = {
                        let mut core = core_ticker.lock().unwrap();
                        core.start_election().ok()
                    };

                    if let Some((args, peers)) = election_data {
                        for (peer_id, peer_addr) in peers {
                            let core_c = core_ticker.clone();
                            let transport_c = transport_ticker.clone();
                            let req = RaftMessage::RequestVote(args.clone());

                            tokio::spawn(async move {
                                if let Ok(RaftMessage::RequestVoteResponse(reply)) =
                                    transport_c.send_rpc(peer_id, &peer_addr, req).await
                                {
                                    let mut core = core_c.lock().unwrap();
                                    let _ = core.handle_vote_reply(reply);
                                }
                            });
                        }
                    }
                }
            }
        });

        // 3. Start Heartbeat & Replication Ticker Task
        let core_heartbeat = self.core.clone();
        let transport_heartbeat = self.transport.clone();
        let running_heartbeat = self.is_running.clone();
        let notify_heartbeat = self.notify_commit.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(40));
            while running_heartbeat.load(Ordering::SeqCst) {
                interval.tick().await;

                let (is_leader, peer_requests) = {
                    let core = core_heartbeat.lock().unwrap();
                    if core.role == RaftRole::Leader {
                        let mut requests = Vec::new();
                        for (&peer_id, peer_addr) in &core.peers {
                            if let Ok(args) = core.create_append_entries_for_peer(peer_id) {
                                requests.push((peer_id, peer_addr.clone(), args));
                            }
                        }
                        (true, requests)
                    } else {
                        (false, Vec::new())
                    }
                };

                if is_leader {
                    for (peer_id, peer_addr, args) in peer_requests {
                        let core_c = core_heartbeat.clone();
                        let transport_c = transport_heartbeat.clone();
                        let notify_c = notify_heartbeat.clone();
                        let req = RaftMessage::AppendEntries(args);

                        tokio::spawn(async move {
                            if let Ok(RaftMessage::AppendEntriesResponse(reply)) =
                                transport_c.send_rpc(peer_id, &peer_addr, req).await
                            {
                                let mut core = core_c.lock().unwrap();
                                if let Ok(newly_committed) = core.handle_append_reply(peer_id, reply) {
                                    if newly_committed {
                                        notify_c.notify_waiters();
                                    }
                                }
                            }
                        });
                    }
                }
            }
        });

        Ok(())
    }

    fn dispatch_rpc(
        core_arc: Arc<Mutex<RaftCore>>,
        notify: Arc<Notify>,
        _my_id: u64,
        msg: RaftMessage,
    ) -> RaftMessage {
        let mut core = core_arc.lock().unwrap();
        match msg {
            RaftMessage::RequestVote(args) => {
                let reply = core
                    .handle_request_vote(args)
                    .unwrap_or(RequestVoteReply {
                        term: core.current_term,
                        vote_granted: false,
                    });
                RaftMessage::RequestVoteResponse(reply)
            }
            RaftMessage::AppendEntries(args) => {
                let reply = core
                    .handle_append_entries(args)
                    .unwrap_or(AppendEntriesReply {
                        term: core.current_term,
                        success: false,
                        match_index: core.last_log_info().0,
                    });
                notify.notify_waiters();
                RaftMessage::AppendEntriesResponse(reply)
            }
            RaftMessage::ClientWrite(args) => {
                match core.client_write(args.command, args.payload) {
                    Ok(index) => RaftMessage::ClientWriteResponse(ClientWriteReply {
                        success: true,
                        index,
                        leader_id: core.leader_id,
                        error: None,
                    }),
                    Err(e) => RaftMessage::ClientWriteResponse(ClientWriteReply {
                        success: false,
                        index: 0,
                        leader_id: core.leader_id,
                        error: Some(e.to_string()),
                    }),
                }
            }
            RaftMessage::ClientRead(args) => {
                match core.read_state_machine(&args.key) {
                    Ok(val) => RaftMessage::ClientReadResponse(ClientReadReply {
                        success: true,
                        value: val,
                        leader_id: core.leader_id,
                        error: None,
                    }),
                    Err(e) => RaftMessage::ClientReadResponse(ClientReadReply {
                        success: false,
                        value: None,
                        leader_id: core.leader_id,
                        error: Some(e.to_string()),
                    }),
                }
            }
            RaftMessage::GetStatus => {
                let (last_idx, _) = core.last_log_info();
                RaftMessage::GetStatusResponse(ClusterStatusReply {
                    node_id: core.node_id,
                    role: core.role,
                    term: core.current_term,
                    leader_id: core.leader_id,
                    commit_index: core.commit_index,
                    last_log_index: last_idx,
                    log_len: last_idx as usize,
                    is_active: true,
                })
            }
            _ => RaftMessage::ClientWriteResponse(ClientWriteReply {
                success: false,
                index: 0,
                leader_id: core.leader_id,
                error: Some("Unhandled message type".to_string()),
            }),
        }
    }

    pub async fn write(&self, command: String, payload: String, timeout: Duration) -> Result<u64, RaftError> {
        let (entry_index, peer_requests) = {
            let mut core = self.core.lock().unwrap();
            if core.role != RaftRole::Leader {
                return Err(RaftError::NotLeader { leader_id: core.leader_id });
            }

            let index = core.client_write(command, payload)?;
            let mut requests = Vec::new();
            for (&peer_id, peer_addr) in &core.peers {
                if let Ok(args) = core.create_append_entries_for_peer(peer_id) {
                    requests.push((peer_id, peer_addr.clone(), args));
                }
            }
            (index, requests)
        };

        // Broadcast replication RPCs immediately
        for (peer_id, peer_addr, args) in peer_requests {
            let core_c = self.core.clone();
            let transport_c = self.transport.clone();
            let notify_c = self.notify_commit.clone();
            let req = RaftMessage::AppendEntries(args);

            tokio::spawn(async move {
                if let Ok(RaftMessage::AppendEntriesResponse(reply)) =
                    transport_c.send_rpc(peer_id, &peer_addr, req).await
                {
                    let mut core = core_c.lock().unwrap();
                    if let Ok(newly_committed) = core.handle_append_reply(peer_id, reply) {
                        if newly_committed {
                            notify_c.notify_waiters();
                        }
                    }
                }
            });
        }

        // Wait for commit progression up to entry_index or timeout
        let start_time = tokio::time::Instant::now();
        loop {
            {
                let core = self.core.lock().unwrap();
                if core.commit_index >= entry_index {
                    return Ok(entry_index);
                }
                if core.role != RaftRole::Leader {
                    return Err(RaftError::NotLeader { leader_id: core.leader_id });
                }
            }

            if start_time.elapsed() > timeout {
                return Err(RaftError::QuorumNotReached {
                    votes: 1,
                    required: (self.core.lock().unwrap().peers.len() + 1) / 2 + 1,
                });
            }

            tokio::select! {
                _ = self.notify_commit.notified() => {},
                _ = tokio::time::sleep(Duration::from_millis(20)) => {},
            }
        }
    }

    pub fn read(&self, key: &str) -> Result<Option<String>, RaftError> {
        let core = self.core.lock().unwrap();
        core.read_state_machine(key)
    }

    pub fn get_status(&self) -> ClusterStatusReply {
        let core = self.core.lock().unwrap();
        let (last_idx, _) = core.last_log_info();
        ClusterStatusReply {
            node_id: core.node_id,
            role: core.role,
            term: core.current_term,
            leader_id: core.leader_id,
            commit_index: core.commit_index,
            last_log_index: last_idx,
            log_len: last_idx as usize,
            is_active: self.is_running.load(Ordering::SeqCst),
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        let _ = self.stop_tx.send(true);
    }
}
