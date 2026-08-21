use super::passport::ProjectPassport;
use serde_json::{json, Value};

pub struct OpenApiGenerator;

impl OpenApiGenerator {
    pub fn generate_openapi_v3(passport: &ProjectPassport) -> Value {
        let mut paths_obj = serde_json::Map::new();
        let mut schemas_obj = serde_json::Map::new();

        // 1. Generate Component Schemas from Structs
        for s in &passport.structs {
            let mut props = serde_json::Map::new();
            let mut required_fields = Vec::new();

            for f in &s.fields {
                let schema_type = Self::map_end_type_to_json_schema(&f.field_type);
                let mut prop_val = schema_type;
                if !f.doc.is_empty() {
                    prop_val["description"] = json!(f.doc);
                }
                props.insert(f.name.clone(), prop_val);
                required_fields.push(json!(f.name));
            }

            schemas_obj.insert(
                s.name.clone(),
                json!({
                    "type": "object",
                    "description": s.doc,
                    "properties": props,
                    "required": required_fields
                }),
            );
        }

        // 2. Generate Component Schemas from Enums
        for e in &passport.enums {
            let variant_names: Vec<Value> = e.variants.iter().map(|v| json!(v.name)).collect();
            schemas_obj.insert(
                e.name.clone(),
                json!({
                    "type": "string",
                    "enum": variant_names,
                    "description": e.doc
                }),
            );
        }

        // 3. Generate Paths & Endpoints
        for ep in &passport.endpoints {
            let method = ep.http_method.to_lowercase();
            let mut operation = json!({
                "tags": [ep.tag],
                "summary": ep.summary,
                "description": if ep.doc.is_empty() { ep.summary.clone() } else { ep.doc.clone() },
                "operationId": ep.handler_name,
                "responses": {
                    "200": {
                        "description": "Successful operation",
                        "content": {
                            "application/json": {
                                "schema": Self::map_response_schema(&ep.response_type)
                            }
                        }
                    },
                    "400": {
                        "description": "Bad request or validation error",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "error": { "type": "string" },
                                        "code": { "type": "integer" }
                                    }
                                }
                            }
                        }
                    },
                    "500": {
                        "description": "Internal server error"
                    }
                }
            });

            if let Some(ref req_body) = ep.request_body_type {
                operation["requestBody"] = json!({
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": Self::map_response_schema(req_body)
                        }
                    }
                });
            }

            if ep.is_authenticated {
                operation["security"] = json!([{ "BearerAuth": [] }]);
            }

            let path_entry = paths_obj.entry(ep.path.clone()).or_insert_with(|| json!({}));
            if let Some(obj) = path_entry.as_object_mut() {
                obj.insert(method, operation);
            }
        }

        // If no endpoints were explicitly tagged, register a default health endpoint
        if paths_obj.is_empty() {
            paths_obj.insert(
                "/health".to_string(),
                json!({
                    "get": {
                        "tags": ["System"],
                        "summary": "Health Check Endpoint",
                        "responses": {
                            "200": {
                                "description": "Service is healthy and active",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "object",
                                            "properties": {
                                                "status": { "type": "string", "example": "UP" },
                                                "version": { "type": "string", "example": passport.metadata.compiler_version }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }),
            );
        }

        json!({
            "openapi": "3.1.0",
            "info": {
                "title": format!("{} API Specification", passport.metadata.name),
                "version": "1.0.0",
                "description": format!("Auto-generated OpenAPI v3.1 Specification by End Language Compiler (v{}).\nHigh-Performance, Zero-Overhead Compiled Backend Service.", passport.metadata.compiler_version),
                "contact": {
                    "name": "End Language Engineering Team",
                    "url": "https://github.com/IrMaho/End"
                }
            },
            "servers": [
                {
                    "url": "http://localhost:8080",
                    "description": "Local Development Server"
                },
                {
                    "url": "https://api.production.internal",
                    "description": "Production High-Performance Cluster"
                }
            ],
            "paths": paths_obj,
            "components": {
                "schemas": schemas_obj,
                "securitySchemes": {
                    "BearerAuth": {
                        "type": "http",
                        "scheme": "bearer",
                        "bearerFormat": "JWT"
                    }
                }
            }
        })
    }

    fn map_end_type_to_json_schema(ty_str: &str) -> Value {
        if ty_str.contains("I8") || ty_str.contains("I16") || ty_str.contains("I32") || ty_str.contains("I64") ||
           ty_str.contains("U8") || ty_str.contains("U16") || ty_str.contains("U32") || ty_str.contains("U64") {
            json!({ "type": "integer", "format": "int64" })
        } else if ty_str.contains("F32") || ty_str.contains("F64") {
            json!({ "type": "number", "format": "double" })
        } else if ty_str.contains("Bool") {
            json!({ "type": "boolean" })
        } else if ty_str.contains("String") || ty_str.contains("str") {
            json!({ "type": "string" })
        } else if ty_str.contains("Array") || ty_str.contains("Vec") {
            json!({ "type": "array", "items": { "type": "string" } })
        } else if let Some(custom) = ty_str.strip_prefix("Custom(\"").and_then(|s| s.strip_suffix("\")")) {
            json!({ "$ref": format!("#/components/schemas/{}", custom) })
        } else {
            json!({ "type": "string" })
        }
    }

    fn map_response_schema(ty_str: &str) -> Value {
        if let Some(custom) = ty_str.strip_prefix("Custom(\"").and_then(|s| s.strip_suffix("\")")) {
            json!({ "$ref": format!("#/components/schemas/{}", custom) })
        } else if ty_str.contains("Array") {
            json!({ "type": "array", "items": { "type": "object" } })
        } else if ty_str == "Void" {
            json!({ "type": "object", "properties": { "status": { "type": "string", "example": "ok" } } })
        } else {
            Self::map_end_type_to_json_schema(ty_str)
        }
    }
}
