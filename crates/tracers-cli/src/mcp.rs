//! MCP server over stdio JSON-RPC.
//!
//! Tools delegate to the `tracers` binary's own subcommands so MCP behaviour
//! always matches the CLI. Configure an MCP client with:
//! `tracers mcp`

use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::process::Command;

const SERVER_INFO: &str = "tracers-mcp";
const PROTOCOL_VERSION: &str = "2024-11-05";

fn arg(params: &Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
}

fn flag(args: &mut Vec<String>, params: &Value, key: &str, name: &str) {
    if arg(params, key).is_some() {
        args.push(format!("--{name}"));
        args.push(arg(params, key).unwrap());
    }
}

fn positional_path(args: &mut Vec<String>, params: &Value) {
    if let Some(path) = arg(params, "path") {
        args.push(path);
    }
}

fn symbol_query(command: &str, params: &Value) -> Vec<String> {
    let mut args = vec![command.to_string()];
    if let Some(symbol) = arg(params, "symbol") {
        args.push(symbol);
    }
    positional_path(&mut args, params);
    flag(&mut args, params, "max_depth", "max-depth");
    flag(&mut args, params, "max_results", "max-results");
    flag(&mut args, params, "graph", "graph");
    args
}

fn target_query(command: &str, params: &Value) -> Vec<String> {
    let mut args = vec![command.to_string()];
    if let Some(target) = arg(params, "target") {
        args.push(target);
    }
    positional_path(&mut args, params);
    if command == "deps" && params.get("transitive").and_then(Value::as_bool) == Some(true) {
        args.push("--transitive".to_string());
    }
    flag(&mut args, params, "max_depth", "max-depth");
    flag(&mut args, params, "max_results", "max-results");
    flag(&mut args, params, "graph", "graph");
    args
}

fn build_args(name: &str, params: &Value) -> Option<Vec<String>> {
    match name {
        "stats" | "cycles" | "validate" | "frameworks" => {
            let mut args = vec![name.to_string()];
            positional_path(&mut args, params);
            flag(&mut args, params, "graph", "graph");
            Some(args)
        }
        "languages" => Some(vec!["languages".to_string()]),
        "roots" => {
            let mut args = vec!["roots".to_string()];
            positional_path(&mut args, params);
            flag(&mut args, params, "max_results", "max-results");
            flag(&mut args, params, "graph", "graph");
            Some(args)
        }
        "graph_export" => {
            let mut args = vec!["graph".to_string()];
            positional_path(&mut args, params);
            flag(&mut args, params, "format", "format");
            flag(&mut args, params, "language", "language");
            flag(&mut args, params, "kind", "kind");
            flag(&mut args, params, "graph", "graph");
            Some(args)
        }
        "path" => {
            let mut args = vec!["path".to_string()];
            if let Some(source) = arg(params, "source") {
                args.push(source);
            }
            if let Some(target) = arg(params, "target") {
                args.push(target);
            }
            positional_path(&mut args, params);
            flag(&mut args, params, "max_depth", "max-depth");
            flag(&mut args, params, "graph", "graph");
            Some(args)
        }
        "callers" | "callees" | "refs" => Some(symbol_query(name, params)),
        "deps" | "impact" | "dependents" => Some(target_query(name, params)),
        _ => None,
    }
}

fn run_cli(args: &[String]) -> Value {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            return json!({
                "content": [{"type": "text", "text": format!("cannot locate tracers binary: {err}")}],
                "isError": true
            })
        }
    };
    match Command::new(exe).args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if output.status.success() {
                let text = if stdout.is_empty() { stderr } else { stdout };
                json!({"content": [{"type": "text", "text": text}]})
            } else {
                json!({
                    "content": [{"type": "text", "text": stderr}],
                    "isError": true
                })
            }
        }
        Err(err) => json!({
            "content": [{"type": "text", "text": format!("failed to run tracers: {err}")}],
            "isError": true
        }),
    }
}

fn schema(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

fn str_prop(description: &str) -> Value {
    json!({"type": "string", "description": description})
}

fn tools_list() -> Value {
    let symbol_props = json!({
        "symbol": str_prop("Symbol selector (ID, qualified name, or short name)"),
        "path": str_prop("Repository path to scan (defaults to current directory)"),
        "max_depth": json!({"type": "integer", "description": "Maximum traversal depth"}),
        "max_results": json!({"type": "integer", "description": "Maximum number of results"}),
        "graph": str_prop("Load the graph from an exported JSON file instead of rescanning")
    });
    let target_props = json!({
        "target": str_prop("Target symbol or file"),
        "path": str_prop("Repository path to scan (defaults to current directory)"),
        "max_depth": json!({"type": "integer", "description": "Maximum traversal depth"}),
        "max_results": json!({"type": "integer", "description": "Maximum number of results"}),
        "graph": str_prop("Load the graph from an exported JSON file instead of rescanning"),
        "transitive": json!({"type": "boolean", "description": "Include transitive dependencies (deps only)"})
    });
    let path_props = json!({
        "path": str_prop("Repository path to scan (defaults to current directory)"),
        "graph": str_prop("Load the graph from an exported JSON file instead of rescanning")
    });
    let tools = json!([
        {"name": "stats", "description": "Report graph statistics for a repository", "inputSchema": schema(path_props.clone(), &[])},
        {"name": "languages", "description": "List supported programming languages", "inputSchema": schema(json!({}), &[])},
        {"name": "frameworks", "description": "Report detected frameworks in the repository", "inputSchema": schema(path_props.clone(), &[])},
        {"name": "validate", "description": "Validate a repository scan and report diagnostics", "inputSchema": schema(path_props.clone(), &[])},
        {"name": "cycles", "description": "Detect dependency cycles in the repository graph", "inputSchema": schema(path_props.clone(), &[])},
        {"name": "roots", "description": "List callable symbols with no incoming call edges", "inputSchema": schema(json!({
            "path": str_prop("Repository path to scan (defaults to current directory)"),
            "max_results": json!({"type": "integer", "description": "Maximum number of results"}),
            "graph": str_prop("Load the graph from an exported JSON file instead of rescanning")
        }), &[])},
        {"name": "graph_export", "description": "Export the repository graph (JSON or DOT)", "inputSchema": schema(json!({
            "path": str_prop("Repository path to scan (defaults to current directory)"),
            "format": str_prop("Output format: json or dot"),
            "language": str_prop("Filter by language"),
            "kind": str_prop("Filter by node kind (comma-separated)"),
            "graph": str_prop("Load the graph from an exported JSON file instead of rescanning")
        }), &[])},
        {"name": "callers", "description": "Display callers of a symbol", "inputSchema": schema(symbol_props.clone(), &["symbol"])},
        {"name": "callees", "description": "Display callees of a symbol", "inputSchema": schema(symbol_props.clone(), &["symbol"])},
        {"name": "refs", "description": "List references to a symbol", "inputSchema": schema(symbol_props.clone(), &["symbol"])},
        {"name": "deps", "description": "Display dependencies of a target", "inputSchema": schema(target_props.clone(), &["target"])},
        {"name": "impact", "description": "Display impact analysis for a target", "inputSchema": schema(target_props.clone(), &["target"])},
        {"name": "dependents", "description": "Display symbols and files that depend on a target", "inputSchema": schema(target_props.clone(), &["target"])},
        {"name": "path", "description": "Find a path between two symbols", "inputSchema": schema(json!({
            "source": str_prop("Source symbol"),
            "target": str_prop("Target symbol"),
            "path": str_prop("Repository path to scan (defaults to current directory)"),
            "max_depth": json!({"type": "integer", "description": "Maximum traversal depth"}),
            "graph": str_prop("Load the graph from an exported JSON file instead of rescanning")
        }), &["source", "target"])}
    ]);
    json!({"tools": tools})
}

fn handle_request(request: &Value) -> Option<Value> {
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let has_id = request.get("id").is_some();

    let result = match method {
        "initialize" => json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": SERVER_INFO, "version": env!("CARGO_PKG_VERSION")}
        }),
        "ping" => Value::Null,
        "tools/list" => tools_list(),
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            match build_args(name, &arguments) {
                Some(args) => run_cli(&args),
                None => json!({
                    "content": [{"type": "text", "text": format!("Unknown tool: {name}")}],
                    "isError": true
                }),
            }
        }
        _ => {
            if !has_id {
                return None;
            }
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("Method not found: {method}")}
            }));
        }
    };

    if !has_id {
        return None;
    }
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

/// Run the MCP server on stdin/stdout until EOF.
pub fn run() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(request) => {
                if let Some(response) = handle_request(&request) {
                    writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                    stdout.flush()?;
                }
            }
            Err(err) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {"code": -32700, "message": format!("Parse error: {err}")}
                });
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
            }
        }
    }
    Ok(())
}
