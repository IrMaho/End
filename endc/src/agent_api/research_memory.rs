use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisEntry {
    pub id: usize,
    pub statement: String,
    pub status: String, // "Investigating", "Confirmed", "Rejected"
    pub conclusion: String,
    pub timestamp_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmEvidence {
    pub evidence_type: String, // "unit_test_log", "compiler_diagnostic", "call_trace", "benchmark"
    pub description: String,
    pub content_hash: String,
    pub payload_snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicResearchMemory {
    pub task_id: String,
    pub requirement: String,
    pub agent_id: String,
    pub created_at_ms: u128,
    pub updated_at_ms: u128,
    pub current_phase: String, // "Exploration", "Hypothesis_Testing", "Implementation", "Verified", "Accepted"
    pub hypotheses: Vec<HypothesisEntry>,
    pub evidence_records: Vec<DrmEvidence>,
    pub investigated_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub contracts_affected: Vec<String>,
    pub architectural_decisions: Vec<String>,
}

pub struct DrmEngine;

impl DrmEngine {
    pub fn memory_dir(project_root: &Path) -> PathBuf {
        project_root.join(".end").join("memory")
    }

    pub fn task_path(project_root: &Path, task_id: &str) -> PathBuf {
        Self::memory_dir(project_root).join(format!("drm_{}.json", task_id))
    }

    pub fn new_task(
        task_id: &str,
        requirement: &str,
        agent_id: &str,
    ) -> DynamicResearchMemory {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        DynamicResearchMemory {
            task_id: task_id.to_string(),
            requirement: requirement.to_string(),
            agent_id: agent_id.to_string(),
            created_at_ms: now,
            updated_at_ms: now,
            current_phase: "Exploration".to_string(),
            hypotheses: Vec::new(),
            evidence_records: Vec::new(),
            investigated_files: Vec::new(),
            modified_files: Vec::new(),
            contracts_affected: Vec::new(),
            architectural_decisions: Vec::new(),
        }
    }

    pub fn save(project_root: &Path, memory: &DynamicResearchMemory) -> Result<PathBuf, String> {
        let dir = Self::memory_dir(project_root);
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create DRM memory dir: {}", e))?;

        let file_path = Self::task_path(project_root, &memory.task_id);
        let json_data = serde_json::to_string_pretty(memory)
            .map_err(|e| format!("Failed to serialize DRM: {}", e))?;

        fs::write(&file_path, json_data)
            .map_err(|e| format!("Failed to write DRM checkpoint: {}", e))?;

        Ok(file_path)
    }

    pub fn load(project_root: &Path, task_id: &str) -> Result<DynamicResearchMemory, String> {
        let file_path = Self::task_path(project_root, task_id);
        if !file_path.exists() {
            return Err(format!("No DRM checkpoint found for task '{}' at {:?}", task_id, file_path));
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed reading DRM file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Corrupted DRM JSON schema: {}", e))
    }

    pub fn list_all_tasks(project_root: &Path) -> Vec<String> {
        let dir = Self::memory_dir(project_root);
        let mut tasks = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("drm_") && name.ends_with(".json") {
                    let task_id = name.trim_start_matches("drm_").trim_end_matches(".json");
                    tasks.push(task_id.to_string());
                }
            }
        }
        tasks
    }
}
