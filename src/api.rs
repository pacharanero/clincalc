// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! HTTP REST API surface for clincalc, behind the `rest-api` feature.
//!
//! Start with `clincalc api [--port 8080] [--host 127.0.0.1]`.
//!
//! Endpoints:
//!   GET  /calculators                    list all calculators
//!   GET  /calculators/{name}/schema      JSON Schema for a calculator's input
//!   GET  /calculators/{name}/template    fillable input template
//!   GET  /calculators/{name}/license     licence and evidence URL
//!   POST /calculators/{name}             compute (body: JSON input object)

use axum::{
    Json, Router,
    extract::{Path, rejection::JsonRejection},
    http::StatusCode,
    routing::{get, post},
};

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<serde_json::Value>)>;

/// Start the axum HTTP server on `host:port`.
pub async fn serve(host: &str, port: u16) -> anyhow::Result<()> {
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("clincalc REST API listening on http://{addr}");
    axum::serve(listener, router()).await?;
    Ok(())
}

fn router() -> Router {
    Router::new()
        .route("/openapi.json", get(get_openapi_spec))
        .route("/calculators", get(list_calculators))
        .route("/calculators/{name}/schema", get(get_schema))
        .route("/calculators/{name}/template", get(get_template))
        .route("/calculators/{name}/license", get(get_license))
        .route("/calculators/{name}", post(compute))
}

fn not_found(name: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": format!("unknown calculator: {name}")})),
    )
}

fn invalid_json(rejection: JsonRejection) -> (StatusCode, Json<serde_json::Value>) {
    (
        rejection.status(),
        Json(serde_json::json!({"error": rejection.body_text()})),
    )
}

async fn list_calculators() -> Json<serde_json::Value> {
    let items: Vec<serde_json::Value> = crate::all()
        .iter()
        .map(|c| {
            let lic = c.license();
            serde_json::json!({
                "name": c.name(),
                "title": c.title(),
                "description": c.description(),
                "supported_locales": c.supported_locales(),
                "license": lic.license,
                "license_source": lic.source_url,
                "tags": c.tags(),
            })
        })
        .collect();
    Json(serde_json::json!(items))
}

async fn get_schema(Path(name): Path<String>) -> ApiResult<serde_json::Value> {
    crate::get(&name)
        .map(|c| Json(c.input_schema()))
        .ok_or_else(|| not_found(&name))
}

async fn get_template(Path(name): Path<String>) -> ApiResult<serde_json::Value> {
    crate::get(&name)
        .map(|c| Json(c.input_template()))
        .ok_or_else(|| not_found(&name))
}

async fn get_license(Path(name): Path<String>) -> ApiResult<serde_json::Value> {
    crate::get(&name)
        .map(|c| Json(serde_json::to_value(c.license()).unwrap()))
        .ok_or_else(|| not_found(&name))
}

async fn compute(
    Path(name): Path<String>,
    payload: Result<Json<serde_json::Value>, JsonRejection>,
) -> ApiResult<serde_json::Value> {
    let calc = crate::get(&name).ok_or_else(|| not_found(&name))?;
    let Json(input) = payload.map_err(invalid_json)?;
    match calc.calculate(&input) {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap())),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": format!("{e}")})),
        )),
    }
}

async fn get_openapi_spec() -> Json<serde_json::Value> {
    Json(openapi_spec())
}

fn openapi_spec() -> serde_json::Value {
    let calcs = crate::all();

    let mut schemas: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    schemas.insert(
        "CalculatorInfo".to_string(),
        serde_json::json!({
            "type": "object",
            "required": ["name", "title", "description", "supported_locales", "license", "license_source", "tags"],
            "properties": {
                "name": {"type": "string"},
                "title": {"type": "string"},
                "description": {"type": "string"},
                "supported_locales": {"type": "array", "items": {"type": "string"}},
                "license": {"type": "string"},
                "license_source": {"type": "string", "format": "uri"},
                "tags": {"type": "array", "items": {"type": "string"}}
            }
        }),
    );

    schemas.insert(
        "CalculationResponse".to_string(),
        serde_json::json!({
            "type": "object",
            "required": ["calculator", "result", "interpretation", "working", "reference"],
            "properties": {
                "calculator": {"type": "string", "description": "Calculator machine name"},
                "result": {"description": "Primary computed value"},
                "interpretation": {"type": "string", "description": "Human-readable clinical interpretation"},
                "working": {"type": "object", "description": "Intermediate values and labels"},
                "reference": {"type": "string", "description": "Citation for the scoring algorithm"}
            }
        }),
    );

    schemas.insert(
        "CalculatorLicense".to_string(),
        serde_json::json!({
            "type": "object",
            "required": ["license", "source_url"],
            "properties": {
                "license": {"type": "string"},
                "source_url": {"type": "string", "format": "uri"}
            }
        }),
    );

    schemas.insert(
        "Error".to_string(),
        serde_json::json!({
            "type": "object",
            "required": ["error"],
            "properties": {"error": {"type": "string"}}
        }),
    );

    for calc in &calcs {
        schemas.insert(format!("Input_{}", calc.name()), calc.input_schema());
    }

    let calculator_names: Vec<serde_json::Value> =
        calcs.iter().map(|c| serde_json::json!(c.name())).collect();

    let name_param = serde_json::json!({
        "name": "name",
        "in": "path",
        "required": true,
        "description": "Calculator machine name (see GET /calculators)",
        "schema": {"type": "string", "enum": calculator_names}
    });

    let error_404 = serde_json::json!({
        "description": "Unknown calculator",
        "content": {"application/json": {"schema": {"$ref": "#/components/schemas/Error"}}}
    });

    let mut paths: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    paths.insert(
        "/calculators".to_string(),
        serde_json::json!({
            "get": {
                "operationId": "listCalculators",
                "summary": "List all calculators",
                "tags": ["catalogue"],
                "responses": {
                    "200": {
                        "description": "Catalogue of calculators",
                        "content": {
                            "application/json": {
                                "schema": {"type": "array", "items": {"$ref": "#/components/schemas/CalculatorInfo"}}
                            }
                        }
                    }
                }
            }
        }),
    );

    paths.insert(
        "/calculators/{name}/schema".to_string(),
        serde_json::json!({
            "get": {
                "operationId": "getSchema",
                "summary": "Get the JSON Schema for a calculator's input",
                "tags": ["calculator"],
                "parameters": [name_param.clone()],
                "responses": {
                    "200": {"description": "JSON Schema object"},
                    "404": error_404.clone()
                }
            }
        }),
    );

    paths.insert(
        "/calculators/{name}/template".to_string(),
        serde_json::json!({
            "get": {
                "operationId": "getTemplate",
                "summary": "Get a fillable input template",
                "tags": ["calculator"],
                "parameters": [name_param.clone()],
                "responses": {
                    "200": {"description": "Template JSON object with placeholder values"},
                    "404": error_404.clone()
                }
            }
        }),
    );

    paths.insert(
        "/calculators/{name}/license".to_string(),
        serde_json::json!({
            "get": {
                "operationId": "getLicense",
                "summary": "Get the distribution licence and evidence URL",
                "tags": ["calculator"],
                "parameters": [name_param],
                "responses": {
                    "200": {
                        "description": "Licence information",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/CalculatorLicense"}
                            }
                        }
                    },
                    "404": error_404.clone()
                }
            }
        }),
    );

    // Per-calculator POST paths: concrete names so Swagger UI shows the correct
    // request body schema for each calculator. Axum routes them all through the
    // single parameterised /calculators/{name} handler.
    for calc in &calcs {
        let schema_ref = format!("#/components/schemas/Input_{}", calc.name());
        paths.insert(
            format!("/calculators/{}", calc.name()),
            serde_json::json!({
                "post": {
                    "operationId": format!("compute_{}", calc.name()),
                    "summary": calc.title(),
                    "description": calc.description(),
                    "tags": calc.tags(),
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {"schema": {"$ref": schema_ref}}
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Calculation result",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/CalculationResponse"}
                                }
                            }
                        },
                        "404": error_404.clone(),
                        "400": {
                            "description": "Malformed JSON request body",
                            "content": {"application/json": {"schema": {"$ref": "#/components/schemas/Error"}}}
                        },
                        "415": {
                            "description": "Request body is not application/json",
                            "content": {"application/json": {"schema": {"$ref": "#/components/schemas/Error"}}}
                        },
                        "422": {
                            "description": "Invalid or incomplete input",
                            "content": {"application/json": {"schema": {"$ref": "#/components/schemas/Error"}}}
                        }
                    }
                }
            }),
        );
    }

    serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "clincalc REST API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Open, auditable clinical calculators. One registry-backed API shape drives every calculator; no per-calculator code required.",
            "license": {
                "name": "AGPL-3.0-or-later",
                "identifier": "AGPL-3.0-or-later"
            },
            "contact": {"url": "https://github.com/pacharanero/clincalc"}
        },
        "paths": serde_json::Value::Object(paths),
        "components": {
            "schemas": serde_json::Value::Object(schemas)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use tower::ServiceExt;

    async fn send(
        router: Router,
        method: Method,
        uri: &str,
        body: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        let request = match body {
            Some(json) => Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(json.to_string()))
                .unwrap(),
            None => Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        };
        send_request(router, request).await
    }

    async fn send_request(
        router: Router,
        request: Request<Body>,
    ) -> (StatusCode, serde_json::Value) {
        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    #[tokio::test]
    async fn list_calculators_returns_all_entries() {
        let (status, body) = send(router(), Method::GET, "/calculators", None).await;
        assert_eq!(status, StatusCode::OK);
        let arr = body.as_array().unwrap();
        assert_eq!(arr.len(), crate::all().len());
        let first = &arr[0];
        assert!(first["name"].is_string());
        assert!(first["title"].is_string());
        assert!(first["description"].is_string());
        assert!(first["supported_locales"].is_array());
        assert!(first["license"].is_string());
        assert!(first["license_source"].is_string());
        assert!(first["tags"].is_array());
    }

    #[tokio::test]
    async fn get_schema_for_known_calculator() {
        let (status, body) =
            send(router(), Method::GET, "/calculators/feverpain/schema", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["title"], "FeverPainInput");
        assert!(body["properties"]["fever"]["type"].is_string());
    }

    #[tokio::test]
    async fn get_schema_for_unknown_calculator_returns_404() {
        let (status, body) = send(router(), Method::GET, "/calculators/nope/schema", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(
            body["error"]
                .as_str()
                .unwrap()
                .contains("unknown calculator: nope")
        );
    }

    #[tokio::test]
    async fn get_template_for_known_calculator() {
        let (status, body) = send(
            router(),
            Method::GET,
            "/calculators/feverpain/template",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.is_object());
        assert!(body["fever"].is_string());
    }

    #[tokio::test]
    async fn get_template_for_unknown_calculator_returns_404() {
        let (status, _body) = send(router(), Method::GET, "/calculators/nope/template", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_license_for_known_calculator() {
        let (status, body) = send(
            router(),
            Method::GET,
            "/calculators/feverpain/license",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body["license"].is_string());
        assert!(body["source_url"].as_str().unwrap().starts_with("http"));
    }

    #[tokio::test]
    async fn get_license_for_unknown_calculator_returns_404() {
        let (status, _body) = send(router(), Method::GET, "/calculators/nope/license", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn compute_valid_input_returns_result() {
        let input = r#"{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":true,"absence_of_cough":true}"#;
        let (status, body) = send(
            router(),
            Method::POST,
            "/calculators/feverpain",
            Some(input),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["calculator"], "feverpain");
        assert_eq!(body["result"], 5);
        assert!(body["interpretation"].is_string());
        assert!(body["working"].is_object());
        assert!(body["reference"].is_string());
    }

    #[tokio::test]
    async fn compute_invalid_input_returns_422() {
        let input = r#"{"fever":"not-a-boolean"}"#;
        let (status, body) = send(
            router(),
            Method::POST,
            "/calculators/feverpain",
            Some(input),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body["error"].as_str().unwrap().contains("invalid input"));
    }

    #[tokio::test]
    async fn malformed_json_returns_json_400() {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/calculators/feverpain")
            .header("content-type", "application/json")
            .body(Body::from("{"))
            .unwrap();
        let (status, body) = send_request(router(), request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body["error"].as_str().unwrap().contains("Failed to parse"));
    }

    #[tokio::test]
    async fn missing_json_content_type_returns_json_415() {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/calculators/feverpain")
            .body(Body::from("{}"))
            .unwrap();
        let (status, body) = send_request(router(), request).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert!(body["error"].as_str().unwrap().contains("Content-Type"));
    }

    #[tokio::test]
    async fn compute_unknown_calculator_returns_404() {
        let input = r#"{}"#;
        let (status, body) = send(router(), Method::POST, "/calculators/nope", Some(input)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(
            body["error"]
                .as_str()
                .unwrap()
                .contains("unknown calculator: nope")
        );
    }

    #[tokio::test]
    async fn openapi_spec_has_all_paths_and_schemas() {
        let (status, body) = send(router(), Method::GET, "/openapi.json", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["openapi"], "3.1.0");
        assert!(body["paths"]["/calculators"]["get"].is_object());
        assert!(body["paths"]["/calculators/{name}/schema"]["get"].is_object());
        assert!(body["paths"]["/calculators/{name}/template"]["get"].is_object());
        assert!(body["paths"]["/calculators/{name}/license"]["get"].is_object());
        assert!(body["paths"]["/calculators/feverpain"]["post"].is_object());
        assert!(body["components"]["schemas"]["CalculatorInfo"].is_object());
        assert!(body["components"]["schemas"]["CalculationResponse"].is_object());
        assert!(body["components"]["schemas"]["CalculatorLicense"].is_object());
        assert!(body["components"]["schemas"]["Error"].is_object());
        assert!(body["components"]["schemas"]["Input_feverpain"].is_object());
        assert!(body["paths"]["/calculators/feverpain"]["post"]["responses"]["400"].is_object());
        assert!(body["paths"]["/calculators/feverpain"]["post"]["responses"]["415"].is_object());
    }

    #[tokio::test]
    async fn openapi_spec_includes_every_calculator_post_path() {
        let (status, body) = send(router(), Method::GET, "/openapi.json", None).await;
        assert_eq!(status, StatusCode::OK);
        for calc in crate::all() {
            let path = format!("/calculators/{}", calc.name());
            assert!(
                body["paths"].get(&path).is_some(),
                "openapi spec missing POST path for {}",
                calc.name()
            );
        }
    }
}
