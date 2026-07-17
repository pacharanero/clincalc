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
    extract::Path,
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

async fn list_calculators() -> Json<serde_json::Value> {
    let items: Vec<serde_json::Value> = crate::all()
        .iter()
        .map(|c| {
            let lic = c.license();
            serde_json::json!({
                "name": c.name(),
                "title": c.title(),
                "description": c.description(),
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
    Json(input): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let calc = crate::get(&name).ok_or_else(|| not_found(&name))?;
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
            "required": ["name", "title", "description", "license", "license_source", "tags"],
            "properties": {
                "name": {"type": "string"},
                "title": {"type": "string"},
                "description": {"type": "string"},
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
