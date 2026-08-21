use super::widget::WidgetNode;
use super::dev_overlay::DevOverlay;

pub struct HtmlUiRenderer;

impl HtmlUiRenderer {
    pub fn render_to_html(root_widget: &WidgetNode, is_dev_mode: bool, feedback_json_str: &str) -> String {
        let widget_dom = Self::render_node(root_widget);
        let overlay_html = if is_dev_mode {
            DevOverlay::get_overlay_script_and_styles(feedback_json_str)
        } else {
            String::new()
        };

        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>EndUI Application</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@300;400;500;600;700;800&family=Fira+Code:wght@400;500;600&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg-body: #07090e;
            --accent-cyan: #06b6d4;
            --accent-indigo: #6366f1;
            --accent-green: #10b981;
            --card-bg: rgba(18, 24, 38, 0.7);
            --border-color: rgba(255, 255, 255, 0.08);
            --text-primary: #f8fafc;
            --text-secondary: #94a3b8;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; font-family: 'Plus Jakarta Sans', sans-serif; }}
        body {{ background: var(--bg-body); color: var(--text-primary); min-height: 100vh; padding: 2rem; display: flex; flex-direction: column; align-items: center; }}
        
        .end-app-container {{ width: 100%; max-width: 1200px; }}
        .end-column {{ display: flex; flex-direction: column; gap: 16px; width: 100%; }}
        .end-row {{ display: flex; flex-direction: row; gap: 16px; align-items: center; width: 100%; }}
        .end-card {{
            background: var(--card-bg); border: 1px solid var(--border-color);
            border-radius: 14px; padding: 24px; backdrop-filter: blur(16px);
            box-shadow: 0 10px 30px rgba(0,0,0,0.3); transition: transform 0.2s, box-shadow 0.2s;
        }}
        .end-card:hover {{ transform: translateY(-2px); box-shadow: 0 15px 35px rgba(99, 102, 241, 0.15); }}
        .end-btn {{
            background: linear-gradient(135deg, var(--accent-cyan), var(--accent-indigo));
            color: #fff; border: none; border-radius: 10px; padding: 12px 24px;
            font-size: 14px; font-weight: 700; cursor: pointer; transition: all 0.2s;
            box-shadow: 0 4px 15px rgba(99, 102, 241, 0.3);
        }}
        .end-btn:hover {{ transform: scale(1.02); box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4); }}
        .end-input {{
            background: rgba(0,0,0,0.4); border: 1px solid var(--border-color);
            border-radius: 8px; padding: 12px 16px; color: #fff; font-size: 14px;
            outline: none; transition: border-color 0.2s; width: 100%;
        }}
        .end-input:focus {{ border-color: var(--accent-indigo); }}
    </style>
</head>
<body>
    <div class="end-app-container">
        {widget_dom}
    </div>
    {overlay_html}
</body>
</html>"#, widget_dom = widget_dom, overlay_html = overlay_html)
    }

    fn render_node(node: &WidgetNode) -> String {
        let data_attrs = format!(
            r#"data-widget-id="{}" data-widget-kind="{}" data-source-file="{}" data-source-line="{}""#,
            node.id, node.kind, node.source_file, node.source_line
        );

        let children_html: String = node.children.iter().map(Self::render_node).collect();

        match node.kind.as_str() {
            "App" => format!(r#"<div class="end-app" {}>{}</div>"#, data_attrs, children_html),
            "Column" => format!(r#"<div class="end-column" {}>{}</div>"#, data_attrs, children_html),
            "Row" => format!(r#"<div class="end-row" {}>{}</div>"#, data_attrs, children_html),
            "Card" => {
                let title = node.properties.get("title").cloned().unwrap_or_default();
                let subtitle = node.properties.get("subtitle").cloned().unwrap_or_default();
                let btn_action = node.properties.get("button_action");

                format!(
                    r#"<div class="end-card" {}>
                        <h3 style="font-size: 18px; font-weight: 700; color: #fff; margin-bottom: 6px;">{}</h3>
                        <p style="font-size: 13px; color: var(--text-secondary); margin-bottom: 12px;">{}</p>
                        {}
                        {}
                    </div>"#,
                    data_attrs,
                    title,
                    subtitle,
                    children_html,
                    btn_action.map(|b| format!(r#"<button class="end-btn" onclick="alert('Action triggered')">{}</button>"#, b)).unwrap_or_default()
                )
            }
            "Text" => {
                let text = node.properties.get("text").cloned().unwrap_or_default();
                let color = node.properties.get("color").cloned().unwrap_or_else(|| "#f8fafc".to_string());
                let sz = node.properties.get("font_size").cloned().unwrap_or_else(|| "14px".to_string());
                let weight = node.properties.get("font_weight").cloned().unwrap_or_else(|| "400".to_string());
                format!(r#"<span style="color: {}; font-size: {}; font-weight: {};" {}>{}</span>"#, color, sz, weight, data_attrs, text)
            }
            "Button" => {
                let text = node.properties.get("text").cloned().unwrap_or_else(|| "Click Me".to_string());
                format!(r#"<button class="end-btn" {} onclick="alert('EndUI Button Clicked!')">{}</button>"#, data_attrs, text)
            }
            "TextField" => {
                let placeholder = node.properties.get("placeholder").cloned().unwrap_or_else(|| "Enter text...".to_string());
                format!(r#"<input type="text" class="end-input" placeholder="{}" {}>"#, placeholder, data_attrs)
            }
            "Container" => format!(r#"<div class="end-container" style="padding: 16px;" {}>{}</div>"#, data_attrs, children_html),
            _ => format!(r#"<div class="end-card" {}><h3>{}</h3>{}</div>"#, data_attrs, node.kind, children_html),
        }
    }
}
