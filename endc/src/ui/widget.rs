use crate::ast::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetNode {
    pub id: String,
    pub kind: String,
    pub properties: HashMap<String, String>,
    pub children: Vec<WidgetNode>,
    pub source_file: String,
    pub source_line: usize,
}

pub struct WidgetTreeExtractor;

impl WidgetTreeExtractor {
    pub fn extract_from_module(module: &Module) -> WidgetNode {
        let app_fn = module.functions.iter().find(|f| {
            f.name == "App" || f.name == "main_widget" || f.directives.iter().any(|d| d.name == "@widget")
        });

        if let Some(f) = app_fn {
            Self::extract_from_function(f, &module.span.file)
        } else {
            Self::create_default_dashboard(module)
        }
    }

    pub fn extract_from_function(func: &FunctionDef, file: &str) -> WidgetNode {
        let mut root = WidgetNode {
            id: format!("widget_root_{}", func.name),
            kind: "App".to_string(),
            properties: HashMap::new(),
            children: Vec::new(),
            source_file: file.to_string(),
            source_line: func.span.line,
        };

        for (idx, stmt) in func.body.statements.iter().enumerate() {
            if let Statement::Return { value: Some(expr), .. } = stmt {
                if let Some(w) = Self::extract_from_expr(expr, file, func.span.line + idx) {
                    root.children.push(w);
                }
            } else if let Statement::Expression(expr) = stmt {
                if let Some(w) = Self::extract_from_expr(expr, file, func.span.line + idx) {
                    root.children.push(w);
                }
            }
        }

        if root.children.is_empty() {
            root.children.push(WidgetNode {
                id: "default_container".to_string(),
                kind: "Container".to_string(),
                properties: [("padding".to_string(), "24px".to_string())].into_iter().collect(),
                children: vec![
                    WidgetNode {
                        id: "default_heading".to_string(),
                        kind: "Text".to_string(),
                        properties: [
                            ("text".to_string(), format!("⚡ EndUI Native Application: {}", func.name)),
                            ("font_size".to_string(), "24px".to_string()),
                            ("font_weight".to_string(), "700".to_string()),
                            ("color".to_string(), "#6366f1".to_string()),
                        ].into_iter().collect(),
                        children: Vec::new(),
                        source_file: file.to_string(),
                        source_line: func.span.line,
                    }
                ],
                source_file: file.to_string(),
                source_line: func.span.line,
            });
        }

        root
    }

    fn extract_from_expr(expr: &Expression, file: &str, line: usize) -> Option<WidgetNode> {
        match expr {
            Expression::StructInit { name, fields, span } => {
                let mut props = HashMap::new();
                let mut children = Vec::new();

                for (fname, fval) in fields {
                    if fname == "child" {
                        if let Some(child_node) = Self::extract_from_expr(fval, file, line) {
                            children.push(child_node);
                        }
                    } else {
                        props.insert(fname.clone(), Self::expr_to_string_val(fval));
                    }
                }

                Some(WidgetNode {
                    id: format!("{}_{}_{}", name, span.line, span.col),
                    kind: name.clone(),
                    properties: props,
                    children,
                    source_file: file.to_string(),
                    source_line: span.line,
                })
            }
            Expression::Call { callee, args, span } => {
                if let Expression::Ident(name, _) = callee.as_ref() {
                    let mut props = HashMap::new();
                    if let Some(first_arg) = args.first() {
                        props.insert("text".to_string(), Self::expr_to_string_val(first_arg));
                    }
                    Some(WidgetNode {
                        id: format!("{}_{}_{}", name, span.line, span.col),
                        kind: name.clone(),
                        properties: props,
                        children: Vec::new(),
                        source_file: file.to_string(),
                        source_line: span.line,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn expr_to_string_val(expr: &Expression) -> String {
        match expr {
            Expression::Lit(Literal::String(s), _) => s.clone(),
            Expression::Lit(Literal::Int(n), _) => n.to_string(),
            Expression::Lit(Literal::Float(f), _) => f.to_string(),
            Expression::Lit(Literal::Bool(b), _) => b.to_string(),
            Expression::Ident(id, _) => id.clone(),
            _ => "dynamic_value".to_string(),
        }
    }

    fn create_default_dashboard(module: &Module) -> WidgetNode {
        let mut children = Vec::new();

        children.push(WidgetNode {
            id: "header_card".to_string(),
            kind: "Card".to_string(),
            properties: [
                ("title".to_string(), format!("🏛️ End Module: {}", module.name)),
                ("subtitle".to_string(), format!("Entry: {} | Functions: {}", module.span.file, module.functions.len())),
                ("bg_color".to_string(), "rgba(99, 102, 241, 0.1)".to_string()),
            ].into_iter().collect(),
            children: Vec::new(),
            source_file: module.span.file.clone(),
            source_line: 1,
        });

        for f in &module.functions {
            children.push(WidgetNode {
                id: format!("fn_widget_{}", f.name),
                kind: "Card".to_string(),
                properties: [
                    ("title".to_string(), format!("⚡ fn {}()", f.name)),
                    ("subtitle".to_string(), format!("Return: {:?}", f.return_type)),
                    ("button_action".to_string(), "Invoke".to_string()),
                ].into_iter().collect(),
                children: Vec::new(),
                source_file: module.span.file.clone(),
                source_line: f.span.line,
            });
        }

        WidgetNode {
            id: "default_app_root".to_string(),
            kind: "App".to_string(),
            properties: HashMap::new(),
            children: vec![
                WidgetNode {
                    id: "main_column".to_string(),
                    kind: "Column".to_string(),
                    properties: [("gap".to_string(), "16px".to_string())].into_iter().collect(),
                    children,
                    source_file: module.span.file.clone(),
                    source_line: 1,
                }
            ],
            source_file: module.span.file.clone(),
            source_line: 1,
        }
    }
}
