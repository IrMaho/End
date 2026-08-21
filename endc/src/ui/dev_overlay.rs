pub struct DevOverlay;

impl DevOverlay {
    pub fn get_overlay_script_and_styles(feedback_json: &str) -> String {
        let template = include_str!("dev_overlay.html");
        template.replace("{feedback_json}", feedback_json)
    }
}
