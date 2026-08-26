use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FrameNode {
    pub name: String,
    pub samples: usize,
    pub children: HashMap<String, FrameNode>,
}

impl FrameNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            samples: 0,
            children: HashMap::new(),
        }
    }

    pub fn insert_stack(&mut self, frames: &[&str], sample_count: usize) {
        self.samples += sample_count;
        if let Some((first, rest)) = frames.split_first() {
            let child = self.children.entry(first.to_string()).or_insert_with(|| FrameNode::new(first));
            child.insert_stack(rest, sample_count);
        }
    }

    pub fn max_depth(&self) -> usize {
        let mut max_child = 0;
        for child in self.children.values() {
            let d = child.max_depth();
            if d > max_child {
                max_child = d;
            }
        }
        1 + max_child
    }
}

pub struct FlameGraphGenerator;

impl FlameGraphGenerator {
    pub fn parse_folded_stacks(folded: &str) -> FrameNode {
        let mut root = FrameNode::new("all");
        for line in folded.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((stack_part, count_part)) = line.rsplit_once(' ') {
                if let Ok(count) = count_part.parse::<usize>() {
                    let frames: Vec<&str> = stack_part.split(';').filter(|s| !s.is_empty()).collect();
                    if !frames.is_empty() {
                        root.insert_stack(&frames, count);
                    }
                }
            }
        }
        root
    }

    pub fn generate_svg(root: &FrameNode, title: &str, total_duration_ms: f64) -> String {
        let total_samples = root.samples.max(1);
        let max_depth = root.max_depth();
        let svg_width = 1200.0;
        let row_height = 24.0;
        let header_height = 60.0;
        let svg_height = header_height + (max_depth as f64) * row_height + 40.0;

        let mut svg = String::new();
        svg.push_str(&format!(
            "<svg viewBox=\"0 0 {} {}\" width=\"100%\" height=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" style=\"font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;\">\n",
            svg_width, svg_height
        ));

        // Background
        svg.push_str(&format!(
            "  <rect width=\"100%\" height=\"100%\" fill=\"#1e1e2e\" />\n"
        ));

        // Header Title
        svg.push_str(&format!(
            "  <text x=\"20\" y=\"30\" fill=\"#cdd6f4\" font-size=\"18\" font-weight=\"bold\">🔥 End Flame Graph: {}</text>\n",
            title
        ));
        svg.push_str(&format!(
            "  <text x=\"20\" y=\"50\" fill=\"#a6adc8\" font-size=\"12\">Total Samples: {} | Runtime: {:.2} ms | Sampling Rate: 1000 Hz</text>\n",
            total_samples, total_duration_ms
        ));

        // Render stack levels recursively
        let mut rects = Vec::new();
        Self::render_node(root, 0.0, svg_width, 0, max_depth, header_height, row_height, total_samples, &mut rects);

        for rect_svg in rects {
            svg.push_str(&rect_svg);
        }

        svg.push_str("</svg>\n");
        svg
    }

    fn render_node(
        node: &FrameNode,
        x: f64,
        width: f64,
        depth: usize,
        max_depth: usize,
        header_height: f64,
        row_height: f64,
        total_samples: usize,
        out: &mut Vec<String>,
    ) {
        if width < 0.5 {
            return;
        }

        // Draw current frame if not the synthetic "all" root
        if depth > 0 {
            let y = header_height + ((max_depth - depth) as f64) * row_height;
            let percent = (node.samples as f64 / total_samples as f64) * 100.0;
            let color = Self::pick_color(&node.name);

            let label = if width > 40.0 {
                let max_chars = ((width - 8.0) / 7.5) as usize;
                if node.name.len() > max_chars && max_chars > 3 {
                    format!("{}...", &node.name[..max_chars - 3])
                } else if node.name.len() <= max_chars {
                    node.name.clone()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            let mut elem = String::new();
            elem.push_str("  <g class=\"func-frame\" cursor=\"pointer\">\n");
            elem.push_str(&format!(
                "    <title>{}: {} samples ({:.2}%)</title>\n",
                node.name, node.samples, percent
            ));
            elem.push_str(&format!(
                "    <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\" stroke=\"#11111b\" stroke-width=\"0.5\" rx=\"2\" />\n",
                x, y, width, row_height - 2.0, color
            ));
            if !label.is_empty() {
                elem.push_str(&format!(
                    "    <text x=\"{:.1}\" y=\"{:.1}\" fill=\"#ffffff\" font-size=\"11\" font-weight=\"500\">{}</text>\n",
                    x + 4.0, y + 14.0, label
                ));
            }
            elem.push_str("  </g>\n");
            out.push(elem);
        }

        // Render children horizontally sorted by sample count
        let mut sorted_children: Vec<&FrameNode> = node.children.values().collect();
        sorted_children.sort_by(|a, b| b.samples.cmp(&a.samples));

        let mut current_x = x;
        for child in sorted_children {
            let child_width = (child.samples as f64 / node.samples as f64) * width;
            Self::render_node(
                child,
                current_x,
                child_width,
                depth + 1,
                max_depth,
                header_height,
                row_height,
                total_samples,
                out,
            );
            current_x += child_width;
        }
    }

    fn pick_color(name: &str) -> &'static str {
        let colors = [
            "#e74c3c", // red
            "#e67e22", // orange
            "#f39c12", // yellow-orange
            "#d35400", // rust
            "#c0392b", // dark red
            "#16a085", // green-cyan
            "#27ae60", // emerald
            "#2980b9", // blue
            "#8e44ad", // purple
            "#f1c40f", // yellow
        ];
        let mut hash: usize = 0;
        for b in name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as usize);
        }
        colors[hash % colors.len()]
    }
}
