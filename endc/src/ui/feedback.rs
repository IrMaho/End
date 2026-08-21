use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use colored::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackItem {
    pub id: String,
    pub target_widget_id: String,
    pub widget_name: String,
    pub source_file: String,
    pub source_line: usize,
    pub developer_note: String,
    pub attached_image: Option<String>, // Base64 data or image URL/path
    pub priority: String, // "P0 (Blocker)", "P1 (High)", "P2 (Normal)"
    pub category: String, // "Design / UI", "Bug", "Task", "Performance"
    pub status: String,   // "Open", "In Progress", "Resolved"
    pub created_at: String,
    pub agent_replies: Vec<AgentReply>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReply {
    pub agent_name: String,
    pub message: String,
    pub timestamp: String,
    pub diff_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskBoard {
    pub project_name: String,
    pub total_tasks: usize,
    pub open_count: usize,
    pub resolved_count: usize,
    pub items: Vec<FeedbackItem>,
}

pub struct FeedbackManager;

impl FeedbackManager {
    pub fn get_feedback_dir(base_dir: &Path) -> PathBuf {
        base_dir.join(".end").join("agent_feedback")
    }

    pub fn list_all(base_dir: &Path) -> Vec<FeedbackItem> {
        let dir = Self::get_feedback_dir(base_dir);
        let mut items = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") && path.file_name().and_then(|s| s.to_str()) != Some("board.json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(item) = serde_json::from_str::<FeedbackItem>(&content) {
                            items.push(item);
                        }
                    }
                }
            }
        }

        // Sort by priority and timestamp
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        items
    }

    pub fn save_item(base_dir: &Path, item: &FeedbackItem) -> Result<PathBuf, String> {
        let dir = Self::get_feedback_dir(base_dir);
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create feedback directory: {}", e))?;

        let file_path = dir.join(format!("{}.json", item.id));
        let json_str = serde_json::to_string_pretty(item).map_err(|e| e.to_string())?;
        fs::write(&file_path, json_str).map_err(|e| format!("Failed to write feedback file: {}", e))?;

        Self::update_board_summary(base_dir)?;
        Ok(file_path)
    }

    pub fn add_reply(
        base_dir: &Path,
        item_id: &str,
        agent_name: &str,
        message: &str,
        new_status: Option<&str>,
    ) -> Result<FeedbackItem, String> {
        let dir = Self::get_feedback_dir(base_dir);
        let file_path = dir.join(format!("{}.json", item_id));

        if !file_path.exists() {
            return Err(format!("Feedback item '{}' not found at {:?}", item_id, file_path));
        }

        let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
        let mut item: FeedbackItem = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        item.agent_replies.push(AgentReply {
            agent_name: agent_name.to_string(),
            message: message.to_string(),
            timestamp: "2026-08-21T21:40:00Z".to_string(),
            diff_file: None,
        });

        if let Some(st) = new_status {
            item.status = st.to_string();
        }

        let updated_json = serde_json::to_string_pretty(&item).map_err(|e| e.to_string())?;
        fs::write(&file_path, updated_json).map_err(|e| e.to_string())?;

        Self::update_board_summary(base_dir)?;
        Ok(item)
    }

    pub fn update_board_summary(base_dir: &Path) -> Result<TaskBoard, String> {
        let items = Self::list_all(base_dir);
        let total = items.len();
        let open_count = items.iter().filter(|i| i.status != "Resolved").count();
        let resolved_count = items.iter().filter(|i| i.status == "Resolved").count();

        let board = TaskBoard {
            project_name: "EndUI Project".to_string(),
            total_tasks: total,
            open_count,
            resolved_count,
            items,
        };

        let dir = Self::get_feedback_dir(base_dir);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let board_json = serde_json::to_string_pretty(&board).map_err(|e| e.to_string())?;
        fs::write(dir.join("board.json"), board_json).map_err(|e| e.to_string())?;

        Ok(board)
    }
}
