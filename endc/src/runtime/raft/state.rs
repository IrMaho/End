use super::error::RaftError;
use super::storage::SqliteRaftStorage;
use super::types::{AppendEntriesArgs, AppendEntriesReply, LogEntry, RaftRole, RequestVoteArgs, RequestVoteReply};
use std::collections::HashMap;
use std::time::Instant;

pub struct RaftCore {
    pub node_id: u64,
    pub peers: HashMap<u64, String>, // peer_id -> "127.0.0.1:port"
    pub storage: SqliteRaftStorage,
    pub current_term: u64,
    pub voted_for: Option<u64>,
    pub role: RaftRole,
    pub leader_id: Option<u64>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub next_index: HashMap<u64, u64>,
    pub match_index: HashMap<u64, u64>,
    pub votes_received: usize,
    pub last_heartbeat_instant: Instant,
}

impl RaftCore {
    pub fn new(node_id: u64, peers: HashMap<u64, String>, storage: SqliteRaftStorage) -> Result<Self, RaftError> {
        let (saved_term, saved_vote) = storage.load_term_and_vote()?;
        let (last_idx, _) = storage.last_log_info()?;

        let mut core = Self {
            node_id,
            peers,
            storage,
            current_term: saved_term,
            voted_for: saved_vote,
            role: RaftRole::Follower,
            leader_id: None,
            commit_index: 0,
            last_applied: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            votes_received: 0,
            last_heartbeat_instant: Instant::now(),
        };

        // If there were existing committed entries, apply them up to last_idx
        if last_idx > 0 {
            core.commit_index = last_idx;
            core.apply_entries_up_to(last_idx)?;
        }

        Ok(core)
    }

    pub fn cluster_size(&self) -> usize {
        self.peers.len() + 1
    }

    pub fn quorum(&self) -> usize {
        (self.cluster_size() / 2) + 1
    }

    pub fn last_log_info(&self) -> (u64, u64) {
        self.storage.last_log_info().unwrap_or((0, 0))
    }

    pub fn become_follower(&mut self, term: u64, leader: Option<u64>) -> Result<(), RaftError> {
        self.role = RaftRole::Follower;
        self.current_term = term;
        self.voted_for = None;
        self.leader_id = leader;
        self.votes_received = 0;
        self.last_heartbeat_instant = Instant::now();
        self.storage.save_term_and_vote(self.current_term, self.voted_for)?;
        Ok(())
    }

    pub fn start_election(&mut self) -> Result<(RequestVoteArgs, Vec<(u64, String)>), RaftError> {
        self.role = RaftRole::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.node_id);
        self.leader_id = None;
        self.votes_received = 1; // Vote for self
        self.last_heartbeat_instant = Instant::now();
        self.storage.save_term_and_vote(self.current_term, self.voted_for)?;

        let (last_idx, last_term) = self.last_log_info();
        let args = RequestVoteArgs {
            term: self.current_term,
            candidate_id: self.node_id,
            last_log_index: last_idx,
            last_log_term: last_term,
        };

        let peer_list: Vec<(u64, String)> = self.peers.iter().map(|(&id, addr)| (id, addr.clone())).collect();
        Ok((args, peer_list))
    }

    pub fn handle_request_vote(&mut self, args: RequestVoteArgs) -> Result<RequestVoteReply, RaftError> {
        if args.term > self.current_term {
            self.become_follower(args.term, None)?;
        }

        if args.term < self.current_term {
            return Ok(RequestVoteReply {
                term: self.current_term,
                vote_granted: false,
            });
        }

        let can_vote = self.voted_for.is_none() || self.voted_for == Some(args.candidate_id);
        let (my_last_idx, my_last_term) = self.last_log_info();

        // Raft log up-to-date comparison (Section 5.4.1)
        let log_ok = if args.last_log_term != my_last_term {
            args.last_log_term > my_last_term
        } else {
            args.last_log_index >= my_last_idx
        };

        if can_vote && log_ok {
            self.voted_for = Some(args.candidate_id);
            self.last_heartbeat_instant = Instant::now();
            self.storage.save_term_and_vote(self.current_term, self.voted_for)?;
            Ok(RequestVoteReply {
                term: self.current_term,
                vote_granted: true,
            })
        } else {
            Ok(RequestVoteReply {
                term: self.current_term,
                vote_granted: false,
            })
        }
    }

    pub fn handle_vote_reply(&mut self, reply: RequestVoteReply) -> Result<bool, RaftError> {
        if reply.term > self.current_term {
            self.become_follower(reply.term, None)?;
            return Ok(false);
        }

        if self.role == RaftRole::Candidate && reply.term == self.current_term && reply.vote_granted {
            self.votes_received += 1;
            if self.votes_received >= self.quorum() {
                self.role = RaftRole::Leader;
                self.leader_id = Some(self.node_id);
                let (last_idx, _) = self.last_log_info();
                self.next_index.clear();
                self.match_index.clear();
                for &peer_id in self.peers.keys() {
                    self.next_index.insert(peer_id, last_idx + 1);
                    self.match_index.insert(peer_id, 0);
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn handle_append_entries(&mut self, args: AppendEntriesArgs) -> Result<AppendEntriesReply, RaftError> {
        if args.term > self.current_term {
            self.become_follower(args.term, Some(args.leader_id))?;
        }

        if args.term < self.current_term {
            return Ok(AppendEntriesReply {
                term: self.current_term,
                success: false,
                match_index: self.last_log_info().0,
            });
        }

        // Recognized valid leader
        if self.role != RaftRole::Follower {
            self.role = RaftRole::Follower;
        }
        self.leader_id = Some(args.leader_id);
        self.last_heartbeat_instant = Instant::now();

        // 2. Reply false if log doesn't contain an entry at prev_log_index matching prev_log_term
        if args.prev_log_index > 0 {
            match self.storage.get_entry(args.prev_log_index)? {
                Some(entry) if entry.term == args.prev_log_term => {}
                _ => {
                    return Ok(AppendEntriesReply {
                        term: self.current_term,
                        success: false,
                        match_index: self.last_log_info().0,
                    });
                }
            }
        }

        // 3. If an existing entry conflicts with a new one, truncate after prev_log_index
        if !args.entries.is_empty() {
            self.storage.truncate_after(args.prev_log_index)?;
            self.storage.append_entries(&args.entries)?;
        }

        // 4. If leader_commit > commit_index, commit_index = min(leader_commit, index of last new entry)
        let (my_last_idx, _) = self.last_log_info();
        if args.leader_commit > self.commit_index {
            let new_commit = args.leader_commit.min(my_last_idx);
            self.apply_entries_up_to(new_commit)?;
            self.commit_index = new_commit;
        }

        Ok(AppendEntriesReply {
            term: self.current_term,
            success: true,
            match_index: my_last_idx,
        })
    }

    pub fn create_append_entries_for_peer(&self, peer_id: u64) -> Result<AppendEntriesArgs, RaftError> {
        let next_idx = *self.next_index.get(&peer_id).unwrap_or(&(self.last_log_info().0 + 1));
        let prev_log_index = if next_idx > 1 { next_idx - 1 } else { 0 };
        let prev_log_term = if prev_log_index > 0 {
            self.storage
                .get_entry(prev_log_index)?
                .map(|e| e.term)
                .unwrap_or(0)
        } else {
            0
        };

        let entries = self.storage.get_entries_from(next_idx)?;

        Ok(AppendEntriesArgs {
            term: self.current_term,
            leader_id: self.node_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit: self.commit_index,
        })
    }

    pub fn handle_append_reply(&mut self, peer_id: u64, reply: AppendEntriesReply) -> Result<bool, RaftError> {
        if reply.term > self.current_term {
            self.become_follower(reply.term, None)?;
            return Ok(false);
        }

        if self.role != RaftRole::Leader || reply.term != self.current_term {
            return Ok(false);
        }

        if reply.success {
            self.match_index.insert(peer_id, reply.match_index);
            self.next_index.insert(peer_id, reply.match_index + 1);

            // Check if there is an N > commit_index such that a majority of match_index[i] >= N and log[N].term == current_term
            let (last_idx, _) = self.last_log_info();
            let mut newly_committed = false;

            for n in (self.commit_index + 1)..=last_idx {
                if let Some(entry) = self.storage.get_entry(n)? {
                    if entry.term == self.current_term {
                        // Count matches
                        let mut count = 1; // self
                        for &m in self.match_index.values() {
                            if m >= n {
                                count += 1;
                            }
                        }
                        if count >= self.quorum() {
                            self.apply_entries_up_to(n)?;
                            self.commit_index = n;
                            newly_committed = true;
                        }
                    }
                }
            }
            Ok(newly_committed)
        } else {
            // Decrement next_index for this peer
            let next_idx = self.next_index.get(&peer_id).copied().unwrap_or(1);
            if next_idx > 1 {
                self.next_index.insert(peer_id, next_idx - 1);
            }
            Ok(false)
        }
    }

    pub fn client_write(&mut self, command: String, payload: String) -> Result<u64, RaftError> {
        if self.role != RaftRole::Leader {
            return Err(RaftError::NotLeader { leader_id: self.leader_id });
        }

        let (last_idx, _) = self.last_log_info();
        let new_index = last_idx + 1;
        let entry = LogEntry {
            index: new_index,
            term: self.current_term,
            command,
            payload,
        };

        self.storage.append_entries(&[entry])?;
        Ok(new_index)
    }

    pub fn apply_entries_up_to(&mut self, target_index: u64) -> Result<(), RaftError> {
        if target_index <= self.last_applied {
            return Ok(());
        }

        let entries = self.storage.get_entries_from(self.last_applied + 1)?;
        for entry in entries {
            if entry.index <= target_index {
                self.storage.apply_to_state_machine(&entry.command, &entry.payload)?;
                self.last_applied = entry.index;
            }
        }
        Ok(())
    }

    pub fn read_state_machine(&self, key: &str) -> Result<Option<String>, RaftError> {
        self.storage.read_state_machine(key)
    }
}
