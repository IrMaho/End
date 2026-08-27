use super::cluster::RaftCluster;
use super::error::RaftError;
use super::storage::SqliteRaftStorage;
use super::types::RaftRole;
use std::time::Duration;

#[tokio::test]
async fn test_01_three_node_startup() {
    let cluster = RaftCluster::start(3, 22100, ":memory:")
        .await
        .expect("Failed to start 3-node cluster");

    assert_eq!(cluster.nodes.len(), 3);
    for i in 1..=3 {
        let status = cluster.get_node_status(i).expect("Node status");
        assert!(status.is_active);
    }
    cluster.stop_all();
}

#[tokio::test]
async fn test_02_leader_election() {
    let cluster = RaftCluster::start(3, 22200, ":memory:")
        .await
        .expect("Failed to start cluster");

    let leader_id = cluster
        .wait_for_leader(Duration::from_secs(4))
        .await
        .expect("Leader should be elected within 4s");

    assert!(leader_id >= 1 && leader_id <= 3);
    let status = cluster.get_node_status(leader_id).unwrap();
    assert_eq!(status.role, RaftRole::Leader);
    assert!(status.term >= 1);

    cluster.stop_all();
}

#[tokio::test]
async fn test_03_and_04_100_entry_replication_and_read_consistency() {
    let cluster = RaftCluster::start(3, 22300, ":memory:")
        .await
        .expect("Failed to start cluster");

    let _leader_id = cluster
        .wait_for_leader(Duration::from_secs(4))
        .await
        .expect("Leader election");

    // Write 100 entries
    let start_time = std::time::Instant::now();
    for i in 1..=100 {
        let key = format!("user_{}", i);
        let payload = format!("key={}&value=cluster_val_{}", key, i * 10);
        let idx = cluster
            .write("SET", &payload)
            .await
            .unwrap_or_else(|e| panic!("Write {} failed: {}", i, e));
        assert_eq!(idx, i);
    }
    let elapsed = start_time.elapsed();
    println!("100 entries replicated and committed in {:?}", elapsed);

    // Wait for follower log flush and verification
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Verify Read Consistency on ALL 3 nodes
    for i in 1..=100 {
        let key = format!("user_{}", i);
        let expected = format!("cluster_val_{}", i * 10);

        for node_id in 1..=3 {
            let val = cluster
                .read_from_node(node_id, &key)
                .expect("Read from node")
                .unwrap_or_else(|| panic!("Key {} missing on node {}", key, node_id));
            assert_eq!(val, expected, "Mismatch on node {} for key {}", node_id, key);
        }
    }

    cluster.stop_all();
}

#[tokio::test]
async fn test_05_and_06_leader_failure_and_post_failure_write() {
    let mut cluster = RaftCluster::start(3, 22400, ":memory:")
        .await
        .expect("Failed to start cluster");

    let old_leader_id = cluster
        .wait_for_leader(Duration::from_secs(4))
        .await
        .expect("First leader");

    // Commit initial write
    cluster.write("SET", "key=leader_test&value=first_round").await.unwrap();

    // Kill Leader (Hard Process Kill Simulation)
    cluster.kill_node(old_leader_id).expect("Killed leader");

    // Wait for new leader election (< 5s)
    let start_elect = std::time::Instant::now();
    let mut new_leader_id = None;
    while start_elect.elapsed() < Duration::from_secs(6) {
        if let Some(lead) = cluster.get_leader_id() {
            if lead != old_leader_id {
                new_leader_id = Some(lead);
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(40)).await;
    }

    let new_lead = new_leader_id.expect("New leader must be elected after leader failure");
    assert_ne!(new_lead, old_leader_id);

    // Post-failure write to new leader
    let new_idx = cluster
        .write("SET", "key=leader_test&value=second_round")
        .await
        .expect("Write to new leader should succeed");
    assert!(new_idx >= 2);

    // Verify read
    let val = cluster.read("leader_test").unwrap();
    assert_eq!(val, Some("second_round".to_string()));

    cluster.stop_all();
}

#[tokio::test]
async fn test_07_and_10_node_restart_sqlite_persistence_and_catch_up() {
    let temp_dir = std::env::temp_dir();
    let db_prefix = temp_dir.join(format!("raft_persist_test_{}", std::process::id()));
    let db_path = db_prefix.to_str().unwrap();

    let mut cluster = RaftCluster::start(3, 22500, db_path)
        .await
        .expect("Failed to start cluster");

    let leader = cluster.wait_for_leader(Duration::from_secs(4)).await.unwrap();

    // Write 5 items
    for i in 1..=5 {
        cluster
            .write("SET", &format!("key=persist_{}&value=val_{}", i, i * 100))
            .await
            .unwrap();
    }

    // Kill Follower Node (e.g. node 3 if leader is not 3, or node 2)
    let follower_id = if leader == 3 { 2 } else { 3 };
    cluster.kill_node(follower_id).unwrap();

    // Write 5 more items while follower is down
    for i in 6..=10 {
        cluster
            .write("SET", &format!("key=persist_{}&value=val_{}", i, i * 100))
            .await
            .unwrap();
    }

    // Restart the node (recovering SQLite persistent state)
    cluster.restart_node(follower_id).await.unwrap();

    // Wait for catch-up with polling
    let mut all_caught_up = false;
    for _ in 0..20 {
        let mut count = 0;
        for i in 1..=10 {
            let key = format!("persist_{}", i);
            let expected = format!("val_{}", i * 100);
            if let Ok(Some(val)) = cluster.read_from_node(follower_id, &key) {
                if val == expected {
                    count += 1;
                }
            }
        }
        if count == 10 {
            all_caught_up = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(all_caught_up, "Restarted node must catch up on all missing items");

    cluster.stop_all();

    // Cleanup DB files
    for i in 1..=3 {
        let file = format!("{}_node_{}.db", db_path, i);
        let _ = std::fs::remove_file(file);
    }
}

#[tokio::test]
async fn test_08_and_09_network_partition_minority_rejection_and_healing() {
    let mut cluster = RaftCluster::start(3, 23600, ":memory:")
        .await
        .expect("Failed to start cluster");

    let leader_id = cluster.wait_for_leader(Duration::from_secs(4)).await.unwrap();

    // Partition the Leader into a minority of 1 (Node A in {A}, vs Majority {B, C})
    let minority_node = leader_id;
    let majority_nodes: Vec<u64> = (1..=3).filter(|&id| id != minority_node).collect();

    cluster.create_partition(&[minority_node], &majority_nodes);

    // 1. Minority node should not be able to commit writes requiring majority quorum
    let minority_server = cluster.nodes.get(&minority_node).unwrap().clone();
    let write_res = minority_server
        .write("SET".to_string(), "key=p_key&value=minority_data".to_string(), Duration::from_millis(300))
        .await;

    assert!(
        write_res.is_err(),
        "Minority node must reject/fail writes that cannot achieve majority quorum"
    );

    // 2. Majority partition {B, C} should elect a new leader and continue making progress!
    let start_maj = std::time::Instant::now();
    let mut new_leader_id = None;
    while start_maj.elapsed() < Duration::from_secs(5) {
        for &id in &majority_nodes {
            let status = cluster.get_node_status(id).unwrap();
            if status.role == RaftRole::Leader {
                new_leader_id = Some(id);
                break;
            }
        }
        if new_leader_id.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(40)).await;
    }

    let new_leader = new_leader_id.expect("Majority partition must elect a new leader");
    assert!(majority_nodes.contains(&new_leader));

    // Write to majority leader
    let maj_server = cluster.nodes.get(&new_leader).unwrap().clone();
    let maj_write = maj_server
        .write("SET".to_string(), "key=p_key&value=majority_data".to_string(), Duration::from_millis(800))
        .await
        .expect("Majority partition must successfully commit writes");
    assert!(maj_write >= 1);

    // 3. Heal Partition
    cluster.heal_partition();

    // Wait for cluster convergence & catch-up with polling
    let mut converged = false;
    for _ in 0..20 {
        let mut count = 0;
        for id in 1..=3 {
            if let Ok(Some(val)) = cluster.read_from_node(id, "p_key") {
                if val == "majority_data" {
                    count += 1;
                }
            }
        }
        if count == 3 {
            converged = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(converged, "All nodes must converge after partition healing");

    cluster.stop_all();
}

#[tokio::test]
async fn test_11_term_safety_and_monotonicity() {
    let mut cluster = RaftCluster::start(3, 23700, ":memory:")
        .await
        .expect("Failed to start cluster");

    let leader_1 = cluster.wait_for_leader(Duration::from_secs(4)).await.unwrap();
    let term_1 = cluster.get_node_status(leader_1).unwrap().term;

    // Kill leader
    cluster.kill_node(leader_1).unwrap();

    // Wait for leader 2
    tokio::time::sleep(Duration::from_millis(500)).await;
    let leader_2 = cluster.wait_for_leader(Duration::from_secs(5)).await.unwrap();
    let term_2 = cluster.get_node_status(leader_2).unwrap().term;

    assert!(term_2 > term_1, "Term must strictly monotonically increase on re-election");

    cluster.stop_all();
}

#[tokio::test]
async fn test_12_no_fake_success_on_disrupted_network() {
    let cluster = RaftCluster::start(3, 22800, ":memory:")
        .await
        .expect("Failed to start cluster");

    let leader = cluster.wait_for_leader(Duration::from_secs(4)).await.unwrap();

    // Isolate leader completely
    cluster.partition_node(leader);

    // Attempt write
    let leader_server = cluster.nodes.get(&leader).unwrap().clone();
    let res = leader_server
        .write("SET".to_string(), "key=k&value=v".to_string(), Duration::from_millis(250))
        .await;

    assert!(res.is_err(), "Must fail when network is disrupted and quorum cannot be reached");

    cluster.stop_all();
}
