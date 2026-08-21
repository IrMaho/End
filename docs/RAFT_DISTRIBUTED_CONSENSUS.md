# 👑 End Language — Distributed Raft Consensus Protocol

> **Zero-Dependency Distributed Consensus Engine in Pure End.**  
> *Build fault-tolerant, replicated state machines, distributed key-value stores, and clustering services with zero external libraries.*

---

## 1. Overview

The **Raft Consensus Protocol** is the gold standard for managing distributed replicated state across multi-node clusters. While other languages require complex C/Go/Rust library wrappers, End includes a high-performance **Raft Consensus Engine** in its standard library (`std/consensus/raft.end`).

```
┌───────────────────────────────────────────────────────────────┐
│                      END RAFT CLUSTER                         │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│       [Node 1] ◄────── RequestVote / Heartbeat ─────► [Node 2]│
│          ▲                                               ▲    │
│          │                                               │    │
│          ▼                                               ▼    │
│       [Node 3] ◄────────────────────────────────────► [Node 4]│
│          ▲                                                    │
│          └──────────────────► [Node 5]                        │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

---

## 2. Standard Library: `std/consensus/raft.end`

### Cluster Initialization & Quorum Calculation

```rust
import "std/consensus/raft.end"

pub fn main() void {
    // 1. Initialize a 5-Node Raft Cluster
    val cluster = raft_create_cluster(5)

    // Calculate Quorum: (5 / 2) + 1 = 3
    val quorum = raft_calculate_quorum(cluster.total_nodes)
    println(quorum) // 3

    // 2. Leader Election Trigger
    val candidate = raft_start_election(cluster.nodes[0])
    println(candidate.term)       // 2
    println(candidate.votes)      // 1 (self-vote)

    // 3. Receive Votes from Peers
    val vote_granted = raft_handle_request_vote(cluster.nodes[1], candidate.node_id, candidate.term)
    if vote_granted {
        println("Vote granted by Follower!")
    }

    // 4. Log Replication via AppendEntries
    val leader = raft_become_leader(candidate)
    val append_ok = raft_append_entries(cluster.nodes[1], leader.current_term, 10, "SET key=42")
    println(append_ok) // true
}
```

---

## 3. Node State Machine & Transitions

The engine enforces strict deterministic state machine invariants:

| State | Role & Responsibilities | Transitions To |
|---|---|---|
| **Follower** | Passive; responds to incoming `RequestVote` and `AppendEntries` RPCs. | **Candidate** (if election timer elapses) |
| **Candidate** | Increments term, votes for self, sends `RequestVote` to all peers. | **Leader** (if quorum votes received) or **Follower** (if higher term discovered) |
| **Leader** | Manages log replication, commits entries, sends periodic heartbeat pings. | **Follower** (if higher term detected) |

---

## 4. Split-Brain & Partition Fault Tolerance

- **Strict Quorum Invariant:** Log commits require acknowledgment from $\lfloor N/2 \rfloor + 1$ nodes. In a 5-node cluster, a 2-node partitioned subset cannot elect a leader or commit entries, preventing split-brain corruption.
- **Microsecond Heartbeats:** With End's sub-millisecond event loop, cluster heartbeats operate at 10-50ms intervals with sub-millisecond failover detection.
