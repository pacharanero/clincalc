// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Feature-gated Model Context Protocol server surface.
//!
//! This module exposes every calculator in [`crate::all`] as an MCP tool. It is
//! compiled only with the optional `mcp` feature so the default-features-off
//! engine remains a serde-only leaf.

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, CallToolResult, ContentBlock, ErrorCode, Implementation, JsonObject,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool, ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::transport::stdio;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, ServiceExt};
use serde_json::Value;

use crate::Calculator;

const TOOL_PREFIX: &str = "clincalc_";

/// Start the local stdio MCP server and wait until the host closes it.
pub async fn serve_stdio() -> anyhow::Result<()> {
    let service = CalcMcpServer.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// Registry-backed MCP server for all calculators.
#[derive(Debug, Clone, Copy, Default)]
pub struct CalcMcpServer;

impl CalcMcpServer {
    fn tools(&self) -> Vec<Tool> {
        crate::all()
            .into_iter()
            .map(|calc| tool_for_calculator(calc.as_ref()))
            .collect()
    }

    fn calculator_for_tool(&self, tool_name: &str) -> Option<Box<dyn Calculator>> {
        crate::all()
            .into_iter()
            .find(|calc| tool_name_for_calculator(calc.name()) == tool_name)
    }
}

impl ServerHandler for CalcMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("clincalc", env!("CARGO_PKG_VERSION"))
                    .with_title("clincalc MCP server")
                    .with_description("Open, auditable clinical calculators exposed as MCP tools"),
            )
            .with_instructions(
                "Each tool is a clinical calculator from the clincalc registry. Provide the required JSON inputs exactly as described by the tool schema; missing or invalid clinical inputs return a validation error.",
            )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: self.tools(),
            next_cursor: None,
            meta: None,
        })
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools().into_iter().find(|tool| tool.name == name)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.call_calculator_tool(&request.name, request.arguments)
    }
}

impl CalcMcpServer {
    fn call_calculator_tool(
        &self,
        tool_name: &str,
        arguments: Option<JsonObject>,
    ) -> Result<CallToolResult, McpError> {
        let calc = self.calculator_for_tool(tool_name).ok_or_else(|| {
            McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("unknown calculator tool: {tool_name}"),
                None,
            )
        })?;

        let input = Value::Object(arguments.unwrap_or_default());
        let response = match calc.calculate(&input) {
            Ok(response) => response,
            Err(err) => {
                return Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                    "{} input was invalid: {err}",
                    calc.name()
                ))]));
            }
        };

        let structured = serde_json::to_value(response).map_err(|err| {
            McpError::internal_error(
                format!("failed to serialise {} response: {err}", calc.name()),
                None,
            )
        })?;

        Ok(CallToolResult::structured(structured))
    }
}

fn tool_for_calculator(calc: &dyn Calculator) -> Tool {
    let lic = calc.license();
    let schema = schema_object(calc.input_schema());
    let description = format!(
        "{} Reference: {} Licence: {} ({})",
        calc.description(),
        calc.reference(),
        lic.license,
        lic.source_url
    );

    Tool::new(
        Cow::Owned(tool_name_for_calculator(calc.name())),
        Cow::Owned(description),
        Arc::new(schema),
    )
    .with_title(calc.title())
    .with_annotations(
        ToolAnnotations::with_title(calc.title())
            .read_only(true)
            .destructive(false)
            .idempotent(true)
            .open_world(false),
    )
}

fn schema_object(schema: Value) -> JsonObject {
    match schema {
        Value::Object(object) => object,
        _ => JsonObject::new(),
    }
}

/// Convert a calculator machine name into a valid MCP tool name.
pub fn tool_name_for_calculator(name: &str) -> String {
    let mut out = String::from(TOOL_PREFIX);
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    #[test]
    fn every_calculator_has_one_tool() {
        let server = CalcMcpServer;
        assert_eq!(server.tools().len(), crate::all().len());
    }

    #[test]
    fn tool_schemas_match_calculator_schemas() {
        let by_name: HashMap<_, _> = CalcMcpServer
            .tools()
            .into_iter()
            .map(|tool| (tool.name.to_string(), tool))
            .collect();

        for calc in crate::all() {
            let tool_name = tool_name_for_calculator(calc.name());
            let tool = by_name
                .get(&tool_name)
                .unwrap_or_else(|| panic!("missing tool for {}", calc.name()));
            assert_eq!(
                Value::Object((*tool.input_schema).clone()),
                calc.input_schema()
            );
        }
    }

    #[test]
    fn tool_names_are_collision_free() {
        let mut seen = HashSet::new();
        for calc in crate::all() {
            let tool_name = tool_name_for_calculator(calc.name());
            assert!(
                seen.insert(tool_name.clone()),
                "duplicate MCP tool name: {tool_name}"
            );
        }
    }

    #[test]
    fn mcp_dispatch_matches_direct_calculation() {
        let server = CalcMcpServer;
        let input = serde_json::json!({
            "fever": true,
            "purulence": true,
            "attend_rapidly": true,
            "inflamed_tonsils": false,
            "absence_of_cough": false
        });

        let direct = crate::get("feverpain")
            .expect("feverpain calculator exists")
            .calculate(&input)
            .expect("direct feverpain calculation succeeds");
        let arguments = input.as_object().expect("input is an object").clone();
        let via_mcp = server
            .call_calculator_tool("clincalc_feverpain", Some(arguments))
            .expect("mcp feverpain calculation succeeds");

        assert_eq!(
            via_mcp.structured_content,
            Some(serde_json::to_value(direct).unwrap())
        );
        assert_eq!(via_mcp.is_error, Some(false));
    }

    #[test]
    fn proprietary_stubs_are_exposed_and_callable() {
        let server = CalcMcpServer;
        let proprietary = crate::all()
            .into_iter()
            .find(|calc| calc.tags().contains(&"proprietary"))
            .expect("at least one proprietary stub is registered");
        let tool_name = tool_name_for_calculator(proprietary.name());

        assert!(server.get_tool(&tool_name).is_some());
        let result = server
            .call_calculator_tool(&tool_name, Some(JsonObject::new()))
            .expect("proprietary stub call succeeds");
        assert_eq!(result.is_error, Some(false));
        assert!(result.structured_content.is_some());
    }
}
