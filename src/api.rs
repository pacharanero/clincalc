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
