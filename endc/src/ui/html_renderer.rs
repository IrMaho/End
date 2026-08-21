use super::widget::WidgetNode;
use super::dev_overlay::DevOverlay;

pub struct HtmlUiRenderer;

impl HtmlUiRenderer {
    pub fn render_to_html(_root_widget: &WidgetNode, is_dev_mode: bool, feedback_json_str: &str) -> String {
        let overlay_html = if is_dev_mode {
            DevOverlay::get_overlay_script_and_styles(feedback_json_str)
        } else {
            String::new()
        };

        let tpl = include_str!("app_template.html");
        tpl.replace("<!-- INJECTED DEVMODE CANVAS OVERLAY -->", &format!("<!-- INJECTED DEVMODE CANVAS OVERLAY -->\n{}", overlay_html))
    }
}
