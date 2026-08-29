// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! HTTP REST API surface for clincalc, behind the `rest-api` feature.
//!
//! Start with `clincalc api [--port 8080] [--host 127.0.0.1] [--locale <tag>]`.
//!
//! Endpoints:
//!   GET  /calculators                    list all calculators
//!   GET  /calculators/{name}/schema      JSON Schema for a calculator's input
//!   GET  /calculators/{name}/template    fillable input template
//!   GET  /calculators/{name}/license     licence and evidence URL
//!   POST /calculators/{name}             compute (body: JSON input object)
//!
//! ## Locale negotiation
//!
//! Every endpoint above except `/license` and `/openapi.json` resolves a
//! locale per RFC 9110 and the contract in `spec/multilingual.md`: an
//! explicit `?locale=<tag>` query parameter takes precedence, then the
//! `Accept-Language` header, then the server's configured default (set via
//! `clincalc api --locale`), then English. An explicit `?locale=` that does
//! not identify an available locale bundle fails with `400`; an
//! `Accept-Language` value that matches nothing quietly falls through to the
//! next tier, matching ordinary content negotiation. On a named calculator
//! endpoint, an explicit locale must be one that calculator advertises.
//! Named calculator responses report the locale actually used via
//! `Content-Language`. The catalogue reports `content_locale` per entry because
//! calculators can fall back independently. Responses add `Vary:
//! Accept-Language` when the header can affect them. The OpenAPI document and
//! licence metadata are not calculator prose and stay locale-neutral.

use std::str::FromStr;

use axum::{
    Json, Router,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::locale::SupportedLocale;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<serde_json::Value>)>;

#[derive(Debug, Clone, Copy)]
struct ApiState {
    default_locale: SupportedLocale,
}

#[derive(Debug, serde::Deserialize)]
struct LocaleQuery {
    locale: Option<String>,
}

/// Start the axum HTTP server on `host:port`, defaulting to English when a
/// request expresses no locale preference of its own.
pub async fn serve(host: &str, port: u16) -> anyhow::Result<()> {
    serve_with_locale(host, port, SupportedLocale::En).await
}

/// Start the axum HTTP server on `host:port` with a configured default locale.
pub async fn serve_with_locale(
    host: &str,
    port: u16,
    default_locale: SupportedLocale,
) -> anyhow::Result<()> {
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("clincalc REST API listening on http://{addr}");
    axum::serve(listener, router(default_locale)).await?;
    Ok(())
}

fn router(default_locale: SupportedLocale) -> Router {
    Router::new()
        .route("/openapi.json", get(get_openapi_spec))
        .route("/calculators", get(list_calculators))
        .route("/calculators/{name}/schema", get(get_schema))
        .route("/calculators/{name}/template", get(get_template))
        .route("/calculators/{name}/license", get(get_license))
        .route("/calculators/{name}", post(compute))
        .with_state(ApiState { default_locale })
}

/// Resolve the locale for one request: explicit query, then `Accept-Language`,
/// then the server default. `available_locales` is the complete locale set for
/// the selected representation. Returns whether `Accept-Language` can affect
/// the response, so callers can emit the required `Vary` header even when the
/// header is absent, malformed, or does not match.
fn negotiate_locale(
    query: &LocaleQuery,
    headers: &HeaderMap,
    default_locale: SupportedLocale,
    available_locales: &[SupportedLocale],
) -> Result<(SupportedLocale, bool), (StatusCode, Json<serde_json::Value>)> {
    if let Some(tag) = query.locale.as_deref() {
        return crate::lookup_locale(tag, available_locales)
            .map(|locale| (locale, false))
            .ok_or_else(|| unsupported_locale_error(tag, available_locales));
    }

    if let Some(header_value) = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
    {
        for range in accept_language_ranges(header_value) {
            if let Some(locale) = crate::lookup_locale(&range, available_locales) {
                return Ok((locale, true));
            }
        }
    }

    let locale = if available_locales.contains(&default_locale) {
        default_locale
    } else {
        SupportedLocale::En
    };
    Ok((locale, true))
}

fn unsupported_locale_error(
    tag: &str,
    available_locales: &[SupportedLocale],
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "error": format!(
                "unsupported locale `{tag}`; available locales: {}",
                available_locales
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })),
    )
}

/// Parse an `Accept-Language` header into language ranges ordered by
/// descending quality (RFC 9110 12.5.4). A range with no explicit `q`
/// defaults to `1.0`. Malformed ranges or `q` values are skipped rather than
/// rejected: an unusable header degrades to the next negotiation tier the
/// same way a header that matches nothing does.
fn accept_language_ranges(header_value: &str) -> Vec<String> {
    let mut ranges: Vec<(String, u32)> = header_value
        .split(',')
        .filter_map(|entry| {
            let mut parts = entry.split(';');
            let range = parts.next()?.trim();
            if range.is_empty() {
                return None;
            }
            let mut quality = 1000;
            let mut has_quality = false;
            for param in parts {
                let (name, value) = param.trim().split_once('=')?;
                if has_quality || !name.trim().eq_ignore_ascii_case("q") {
                    return None;
                }
                quality = parse_quality(value.trim())?;
                has_quality = true;
            }
            if quality == 0 {
                return None;
            }
            Some((range.to_string(), quality))
        })
        .collect();
    // Stable sort: entries with equal quality keep the header's own order.
    ranges.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    ranges.into_iter().map(|(range, _)| range).collect()
}

/// Parse an RFC 9110 `qvalue` (`0` to `1`, up to three decimal digits) into a
/// fixed-point integer out of 1000, so ranges sort without float comparison.
fn parse_quality(value: &str) -> Option<u32> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if fraction.len() > 3 || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    match whole {
        "0" => {
            let fraction = if fraction.is_empty() {
                0
            } else {
                fraction.parse::<u32>().ok()? * 10_u32.pow(3 - fraction.len() as u32)
            };
            Some(fraction)
        }
        "1" if fraction.bytes().all(|byte| byte == b'0') => Some(1000),
        _ => None,
    }
}

fn add_vary_accept_language(headers: &mut HeaderMap) {
    let already_present = headers
        .get_all(header::VARY)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|name| name.trim().eq_ignore_ascii_case("Accept-Language"));
    if !already_present {
        headers.append(header::VARY, HeaderValue::from_static("Accept-Language"));
    }
}

/// Attach `Content-Language` (and `Vary: Accept-Language` when negotiated) to
/// a JSON response.
fn with_locale_headers(
    json: Json<serde_json::Value>,
    locale: SupportedLocale,
    varies_by_accept_language: bool,
) -> Response {
    let mut response = json.into_response();
    response.headers_mut().insert(
        header::CONTENT_LANGUAGE,
        HeaderValue::from_static(locale.as_bcp47()),
    );
    if varies_by_accept_language {
        add_vary_accept_language(response.headers_mut());
    }
    response
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

async fn list_calculators(
    State(state): State<ApiState>,
    Query(query): Query<LocaleQuery>,
    headers: HeaderMap,
) -> Response {
    let (locale, negotiated) = match negotiate_locale(
        &query,
        &headers,
        state.default_locale,
        crate::COMPILED_LOCALES,
    ) {
        Ok(resolved) => resolved,
        Err(err) => return err.into_response(),
    };
    let items: Vec<serde_json::Value> = crate::all()
        .iter()
        .map(|c| {
            let lic = c.license();
            let content_locale = if c.supported_locales().contains(&locale) {
                locale
            } else {
                SupportedLocale::En
            };
            serde_json::json!({
                "name": c.name(),
                "title": c.title_for(content_locale),
                "description": c.description_for(content_locale),
                "content_locale": content_locale,
                "supported_locales": c.supported_locales(),
                "license": lic.license,
                "license_source": lic.source_url,
                "tags": c.tags(),
            })
        })
        .collect();
    // No single Content-Language: the catalogue mixes calculators that may
    // each fall back to English independently, so every item reports its own.
    let mut response = Json(serde_json::json!(items)).into_response();
    if negotiated {
        add_vary_accept_language(response.headers_mut());
    }
    response
}

async fn get_schema(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Query(query): Query<LocaleQuery>,
    headers: HeaderMap,
) -> Response {
    let Some(calc) = crate::get(&name) else {
        return not_found(&name).into_response();
    };
    let (locale, negotiated) = match negotiate_locale(
        &query,
        &headers,
        state.default_locale,
        calc.supported_locales(),
    ) {
        Ok(resolved) => resolved,
        Err(err) => return err.into_response(),
    };
    with_locale_headers(Json(calc.input_schema_for(locale)), locale, negotiated)
}

async fn get_template(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Query(query): Query<LocaleQuery>,
    headers: HeaderMap,
) -> Response {
    let Some(calc) = crate::get(&name) else {
        return not_found(&name).into_response();
    };
    let (locale, negotiated) = match negotiate_locale(
        &query,
        &headers,
        state.default_locale,
        calc.supported_locales(),
    ) {
        Ok(resolved) => resolved,
        Err(err) => return err.into_response(),
    };
    with_locale_headers(Json(calc.input_template_for(locale)), locale, negotiated)
}

async fn get_license(Path(name): Path<String>) -> ApiResult<serde_json::Value> {
    crate::get(&name)
        .map(|c| Json(serde_json::to_value(c.license()).unwrap()))
        .ok_or_else(|| not_found(&name))
}

async fn compute(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Query(query): Query<LocaleQuery>,
    headers: HeaderMap,
    payload: Result<Json<serde_json::Value>, JsonRejection>,
) -> Response {
    let Some(calc) = crate::get(&name) else {
        return not_found(&name).into_response();
    };
    let (locale, negotiated) = match negotiate_locale(
        &query,
        &headers,
        state.default_locale,
        calc.supported_locales(),
    ) {
        Ok(resolved) => resolved,
        Err(err) => return err.into_response(),
    };
    let input = match payload {
        Ok(Json(input)) => input,
        Err(rejection) => return invalid_json(rejection).into_response(),
    };
    match calc.calculate_for(&input, locale) {
        Ok(response) => {
            // Ground truth: `calculate_for` stamps the bundle it actually
            // rendered in `working.content_locale`, which may differ from
            // the requested/negotiated locale if this calculator lacks it.
            let content_locale = response
                .working
                .get("content_locale")
                .and_then(|value| value.as_str())
                .and_then(|tag| SupportedLocale::from_str(tag).ok())
                .unwrap_or(locale);
            with_locale_headers(
                Json(serde_json::to_value(response).unwrap()),
                content_locale,
                negotiated,
            )
        }
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": format!("{e}")})),
        )
            .into_response(),
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
            "required": ["name", "title", "description", "content_locale", "supported_locales", "license", "license_source", "tags"],
            "properties": {
                "name": {"type": "string"},
                "title": {"type": "string"},
                "description": {"type": "string"},
                "content_locale": {"type": "string", "description": "Canonical BCP 47 tag of the complete locale bundle used for this entry"},
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

    let locale_query_param = serde_json::json!({
        "name": "locale",
        "in": "query",
        "required": false,
        "description": "Explicit BCP 47 locale tag. On named calculator endpoints, the locale must be listed in that calculator's supported_locales.",
        "schema": {"type": "string"}
    });

    let accept_language_param = serde_json::json!({
        "name": "Accept-Language",
        "in": "header",
        "required": false,
        "description": "RFC 9110 language preferences, used when locale is not supplied.",
        "schema": {"type": "string"}
    });

    let catalogue_locale_params =
        serde_json::json!([locale_query_param.clone(), accept_language_param.clone()]);
    let calculator_locale_params = serde_json::json!([
        name_param.clone(),
        locale_query_param,
        accept_language_param
    ]);

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
                "parameters": catalogue_locale_params,
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
                "parameters": calculator_locale_params.clone(),
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
                "parameters": calculator_locale_params.clone(),
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
                    "parameters": calculator_locale_params.clone(),
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
                "name": "AGPL-3.0-or-later AND LGPL-3.0-or-later",
                "identifier": "AGPL-3.0-or-later AND LGPL-3.0-or-later"
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
        let (status, body) = send(
            router(SupportedLocale::En),
            Method::GET,
            "/calculators",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let arr = body.as_array().unwrap();
        assert_eq!(arr.len(), crate::all().len());
        let first = &arr[0];
        assert!(first["name"].is_string());
        assert!(first["title"].is_string());
        assert!(first["description"].is_string());
        assert!(first["content_locale"].is_string());
        assert!(first["supported_locales"].is_array());
        assert!(first["license"].is_string());
        assert!(first["license_source"].is_string());
        assert!(first["tags"].is_array());
    }

    #[tokio::test]
    async fn get_schema_for_known_calculator() {
        let (status, body) = send(
            router(SupportedLocale::En),
            Method::GET,
            "/calculators/feverpain/schema",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["title"], "FeverPainInput");
        assert!(body["properties"]["fever"]["type"].is_string());
    }

    #[tokio::test]
    async fn get_schema_for_unknown_calculator_returns_404() {
        let (status, body) = send(
            router(SupportedLocale::En),
            Method::GET,
            "/calculators/nope/schema",
            None,
        )
        .await;
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
            router(SupportedLocale::En),
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
        let (status, _body) = send(
            router(SupportedLocale::En),
            Method::GET,
            "/calculators/nope/template",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_license_for_known_calculator() {
        let (status, body) = send(
            router(SupportedLocale::En),
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
        let (status, _body) = send(
            router(SupportedLocale::En),
            Method::GET,
            "/calculators/nope/license",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn compute_valid_input_returns_result() {
        let input = r#"{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":true,"absence_of_cough":true}"#;
        let (status, body) = send(
            router(SupportedLocale::En),
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
            router(SupportedLocale::En),
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
        let (status, body) = send_request(router(SupportedLocale::En), request).await;
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
        let (status, body) = send_request(router(SupportedLocale::En), request).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert!(body["error"].as_str().unwrap().contains("Content-Type"));
    }

    #[tokio::test]
    async fn compute_unknown_calculator_returns_404() {
        let input = r#"{}"#;
        let (status, body) = send(
            router(SupportedLocale::En),
            Method::POST,
            "/calculators/nope",
            Some(input),
        )
        .await;
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
        let (status, body) = send(
            router(SupportedLocale::En),
            Method::GET,
            "/openapi.json",
            None,
        )
        .await;
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
        assert_eq!(
            body["paths"]["/calculators"]["get"]["parameters"][0]["name"],
            "locale"
        );
        assert_eq!(
            body["paths"]["/calculators/feverpain"]["post"]["parameters"][2]["name"],
            "Accept-Language"
        );
    }

    #[tokio::test]
    async fn openapi_spec_includes_every_calculator_post_path() {
        let (status, body) = send(
            router(SupportedLocale::En),
            Method::GET,
            "/openapi.json",
            None,
        )
        .await;
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

    // --- Locale negotiation ---------------------------------------------

    #[test]
    fn negotiate_locale_prefers_explicit_query() {
        let query = LocaleQuery {
            locale: Some("es".to_string()),
        };
        let (locale, negotiated) = negotiate_locale(
            &query,
            &HeaderMap::new(),
            SupportedLocale::En,
            crate::COMPILED_LOCALES,
        )
        .unwrap();
        assert_eq!(locale, SupportedLocale::Es);
        assert!(!negotiated);
    }

    #[test]
    fn negotiate_locale_rejects_an_unrecognised_explicit_query() {
        let query = LocaleQuery {
            locale: Some("xx".to_string()),
        };
        let (status, body) = negotiate_locale(
            &query,
            &HeaderMap::new(),
            SupportedLocale::En,
            crate::COMPILED_LOCALES,
        )
        .unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            body.0["error"]
                .as_str()
                .unwrap()
                .contains("unsupported locale `xx`")
        );
    }

    #[test]
    fn negotiate_locale_falls_back_to_accept_language_header() {
        let query = LocaleQuery { locale: None };
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static("fr;q=0.5, ca;q=0.9, en;q=0.1"),
        );
        let (locale, negotiated) = negotiate_locale(
            &query,
            &headers,
            SupportedLocale::En,
            crate::COMPILED_LOCALES,
        )
        .unwrap();
        assert_eq!(locale, SupportedLocale::Ca);
        assert!(negotiated);
    }

    #[test]
    fn negotiate_locale_ignores_a_header_that_matches_no_compiled_bundle() {
        let query = LocaleQuery { locale: None };
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_LANGUAGE, HeaderValue::from_static("de, fr"));
        let (locale, negotiated) = negotiate_locale(
            &query,
            &headers,
            SupportedLocale::Es,
            crate::COMPILED_LOCALES,
        )
        .unwrap();
        assert_eq!(locale, SupportedLocale::Es);
        assert!(negotiated);
    }

    #[test]
    fn negotiate_locale_uses_the_server_default_with_no_request_signal() {
        let query = LocaleQuery { locale: None };
        let (locale, negotiated) = negotiate_locale(
            &query,
            &HeaderMap::new(),
            SupportedLocale::Ca,
            crate::COMPILED_LOCALES,
        )
        .unwrap();
        assert_eq!(locale, SupportedLocale::Ca);
        assert!(negotiated);
    }

    #[test]
    fn accept_language_ranges_sort_by_descending_quality_and_keep_header_order_for_ties() {
        assert_eq!(
            accept_language_ranges("en;q=0.5, es, ca;q=0.5, fr;q=0.9"),
            vec!["es", "fr", "en", "ca"]
        );
    }

    #[test]
    fn accept_language_ranges_skips_malformed_and_zero_quality_ranges() {
        assert_eq!(
            accept_language_ranges("es;q=notanumber, ca;q=1.001, fr;q=0, en;q=0.9, de;q=0.1234"),
            vec!["en"]
        );
    }

    #[test]
    fn parse_quality_accepts_only_the_rfc_qvalue_grammar() {
        assert_eq!(parse_quality("0"), Some(0));
        assert_eq!(parse_quality("0.5"), Some(500));
        assert_eq!(parse_quality("0.125"), Some(125));
        assert_eq!(parse_quality("1.000"), Some(1000));
        assert_eq!(parse_quality("1.001"), None);
        assert_eq!(parse_quality("0.1234"), None);
        assert_eq!(parse_quality(".5"), None);
        assert_eq!(parse_quality("-0.5"), None);
    }

    #[test]
    fn negotiate_locale_uses_the_selected_calculators_available_locales() {
        let query = LocaleQuery { locale: None };
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static("es, ca;q=0.8, en;q=0.5"),
        );
        let available = [SupportedLocale::En, SupportedLocale::Ca];
        let (locale, negotiated) =
            negotiate_locale(&query, &headers, SupportedLocale::En, &available).unwrap();
        assert_eq!(locale, SupportedLocale::Ca);
        assert!(negotiated);
    }

    #[test]
    fn add_vary_accept_language_preserves_existing_values() {
        let mut headers = HeaderMap::new();
        headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
        add_vary_accept_language(&mut headers);
        let values: Vec<_> = headers
            .get_all(header::VARY)
            .iter()
            .map(|value| value.to_str().unwrap())
            .collect();
        assert_eq!(values, vec!["Accept-Encoding", "Accept-Language"]);
    }

    #[test]
    fn accept_language_ranges_skips_blank_entries() {
        assert_eq!(accept_language_ranges("es,, en"), vec!["es", "en"]);
    }

    #[tokio::test]
    async fn get_schema_with_an_unrecognised_explicit_locale_returns_400() {
        let (status, body) = send(
            router(SupportedLocale::En),
            Method::GET,
            "/calculators/feverpain/schema?locale=xx",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            body["error"]
                .as_str()
                .unwrap()
                .contains("unsupported locale `xx`")
        );
    }

    #[tokio::test]
    async fn get_schema_rejects_a_locale_the_calculator_does_not_support() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/calculators/feverpain/schema?locale=es")
            .body(Body::empty())
            .unwrap();
        let response = router(SupportedLocale::En).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(response.headers().get(header::VARY).is_none());
    }

    #[tokio::test]
    async fn schema_without_an_explicit_locale_sets_vary_even_without_a_header() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/calculators/feverpain/schema")
            .body(Body::empty())
            .unwrap();
        let response = router(SupportedLocale::En).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::VARY).unwrap(),
            "Accept-Language"
        );
    }

    #[tokio::test]
    async fn compute_negotiated_via_accept_language_sets_vary() {
        let input = r#"{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":true,"absence_of_cough":true}"#;
        let request = Request::builder()
            .method(Method::POST)
            .uri("/calculators/feverpain")
            .header("content-type", "application/json")
            .header("accept-language", "es")
            .body(Body::from(input))
            .unwrap();
        let response = router(SupportedLocale::En).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_LANGUAGE).unwrap(),
            "en"
        );
        assert_eq!(
            response.headers().get(header::VARY).unwrap(),
            "Accept-Language"
        );
    }

    #[tokio::test]
    async fn compute_reports_content_locale_in_working() {
        let input = r#"{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":true,"absence_of_cough":true}"#;
        let (status, body) = send(
            router(SupportedLocale::En),
            Method::POST,
            "/calculators/feverpain",
            Some(input),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["working"]["content_locale"], "en");
    }

    #[tokio::test]
    async fn list_calculators_accepts_a_compiled_but_unsupported_locale() {
        let (status, body) = send(
            router(SupportedLocale::En),
            Method::GET,
            "/calculators?locale=es",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let first = &body.as_array().unwrap()[0];
        assert!(first["title"].is_string());
        assert_eq!(first["content_locale"], "en");
    }
}
