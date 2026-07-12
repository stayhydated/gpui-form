use std::{
    io::{BufRead as _, BufReader, Write as _},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::{Value, json};

struct StdioMcp {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl StdioMcp {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_mcp-submit"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("mcp-submit binary should spawn");
        let stdin = child.stdin.take().expect("stdin should be piped");
        let stdout = BufReader::new(child.stdout.take().expect("stdout should be piped"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        writeln!(self.stdin, "{request}").expect("request should write");
        self.stdin.flush().expect("request should flush");

        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .expect("response should read");
        assert!(!line.is_empty(), "server closed stdout before response");
        let response: Value = serde_json::from_str(&line).expect("response should be JSON");
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], json!(id));
        response
    }

    fn notify(&mut self, method: &str, params: Value) {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        writeln!(self.stdin, "{notification}").expect("notification should write");
        self.stdin.flush().expect("notification should flush");
    }

    fn shutdown(mut self) {
        drop(self.stdin);
        let status = self.child.wait().expect("server should exit");
        assert!(status.success(), "server exited with {status}");
    }
}

#[test]
fn stdio_json_rpc_lists_calls_repairs_and_submits_edit_sessions() {
    let mut server = StdioMcp::spawn();

    let initialized = server.request(
        0,
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {
                "name": "stdio-test",
                "version": "0.0.0"
            }
        }),
    );
    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(initialized["result"]["capabilities"]["tools"], json!({}));
    assert_eq!(
        initialized["result"]["capabilities"]["resources"],
        json!({})
    );
    assert_eq!(initialized["result"]["capabilities"]["prompts"], json!({}));
    server.notify("notifications/initialized", json!({}));

    let listed = server.request(1, "tools/list", json!({}));
    let tools = listed["result"]["tools"]
        .as_array()
        .expect("tools/list should return tools");
    let support_tool = tools
        .iter()
        .find(|tool| tool["name"] == "mcp_submit_support_ticket")
        .expect("support submit tool should be listed");
    assert_eq!(
        support_tool["inputSchema"]["properties"]["title"]["type"],
        "string"
    );
    assert_eq!(
        support_tool["inputSchema"]["properties"]["title"]["title"],
        "Title"
    );
    assert_eq!(support_tool["outputSchema"]["type"], "object");
    assert!(
        tools
            .iter()
            .any(|tool| tool["name"] == "mcp_submit_support_ticket_edit_open")
    );
    assert!(
        tools
            .iter()
            .any(|tool| tool["name"] == "mcp_submit_support_ticket_edit_submit")
    );

    let listed_resources = server.request(20, "resources/list", json!({}));
    let resources = listed_resources["result"]["resources"]
        .as_array()
        .expect("resources/list should return resources");
    let descriptor_uri = "gpui-form://forms/mcp_submit_support_ticket/descriptor";
    let schema_uri = "gpui-form://forms/mcp_submit_support_ticket/schema";
    assert!(
        resources
            .iter()
            .any(|resource| resource["uri"] == descriptor_uri
                && resource["mimeType"] == "application/json")
    );
    assert!(resources.iter().any(
        |resource| resource["uri"] == schema_uri && resource["mimeType"] == "application/json"
    ));

    let descriptor_resource = server.request(
        21,
        "resources/read",
        json!({
            "uri": descriptor_uri
        }),
    );
    let descriptor_content = &descriptor_resource["result"]["contents"][0];
    assert_eq!(descriptor_content["uri"], descriptor_uri);
    assert_eq!(descriptor_content["mimeType"], "application/json");
    let descriptor: Value = serde_json::from_str(
        descriptor_content["text"]
            .as_str()
            .expect("descriptor resource should contain text"),
    )
    .expect("descriptor resource should contain JSON");
    assert_eq!(descriptor["tool_name"], "mcp_submit_support_ticket");
    assert_eq!(descriptor["title"], "Submit support ticket");
    assert_eq!(descriptor["fields"][0]["name"], "title");
    assert_eq!(
        descriptor["fields"][0]["validation_rules"][0]["validator"],
        "NonEmptyValidation"
    );

    let schema_resource = server.request(
        22,
        "resources/read",
        json!({
            "uri": schema_uri
        }),
    );
    let schema_content = &schema_resource["result"]["contents"][0];
    let schema: Value = serde_json::from_str(
        schema_content["text"]
            .as_str()
            .expect("schema resource should contain text"),
    )
    .expect("schema resource should contain JSON");
    assert_eq!(schema["properties"]["title"]["type"], "string");
    assert_eq!(schema["properties"]["title"]["minLength"], 1);

    let listed_prompts = server.request(23, "prompts/list", json!({}));
    let prompts = listed_prompts["result"]["prompts"]
        .as_array()
        .expect("prompts/list should return prompts");
    let fill_prompt = prompts
        .iter()
        .find(|prompt| prompt["name"] == "fill_mcp_submit_support_ticket_form")
        .expect("support fill prompt should be listed");
    assert_eq!(fill_prompt["title"], "Fill Submit support ticket");
    assert_eq!(fill_prompt["arguments"][0]["name"], "goal");
    assert!(
        prompts
            .iter()
            .any(|prompt| prompt["name"] == "repair_mcp_submit_support_ticket_form")
    );
    assert!(
        prompts
            .iter()
            .any(|prompt| prompt["name"] == "submit_mcp_submit_support_ticket_form")
    );

    let repair_prompt = server.request(
        24,
        "prompts/get",
        json!({
            "name": "repair_mcp_submit_support_ticket_form",
            "arguments": {
                "validation_issues": [
                    { "field": "title", "message": "must not be empty" }
                ],
                "current_values": {
                    "title": ""
                }
            }
        }),
    );
    assert_eq!(
        repair_prompt["result"]["description"],
        "Repair Submit support ticket."
    );
    assert_eq!(repair_prompt["result"]["messages"][0]["role"], "user");
    let repair_prompt_text = repair_prompt["result"]["messages"][0]["content"]["text"]
        .as_str()
        .expect("repair prompt should contain text content");
    assert!(repair_prompt_text.contains(descriptor_uri));
    assert!(repair_prompt_text.contains(schema_uri));
    assert!(repair_prompt_text.contains("mcp_submit_support_ticket_edit_patch"));
    assert!(repair_prompt_text.contains("Caller-provided context"));

    let submitted = server.request(
        2,
        "tools/call",
        json!({
            "name": "mcp_submit_support_ticket",
            "arguments": {
                "title": "Cannot log in",
                "details": "SSO loops back to the login page",
                "urgent": true
            }
        }),
    );
    assert_eq!(submitted["result"]["isError"], false);
    assert_eq!(submitted["result"]["structuredContent"]["accepted"], true);
    assert_eq!(
        submitted["result"]["structuredContent"]["title"],
        "Cannot log in"
    );

    let validation = server.request(
        3,
        "tools/call",
        json!({
            "name": "mcp_submit_support_ticket",
            "arguments": {
                "title": ""
            }
        }),
    );
    assert_eq!(validation["result"]["isError"], true);
    assert_eq!(
        validation["result"]["structuredContent"]["error"]["kind"],
        "validation"
    );
    assert!(
        validation["result"]["structuredContent"]["error"]["details"]
            .as_array()
            .is_some_and(|details| !details.is_empty())
    );

    let opened = server.request(
        4,
        "tools/call",
        json!({
            "name": "mcp_submit_support_ticket_edit_open",
            "arguments": {
                "values": {
                    "title": "Draft ticket"
                }
            }
        }),
    );
    assert_eq!(opened["result"]["isError"], false);
    let session_id = opened["result"]["structuredContent"]["session_id"]
        .as_str()
        .expect("edit_open should return session id")
        .to_string();
    assert_eq!(
        opened["result"]["structuredContent"]["submit_arguments"],
        json!({ "title": "Draft ticket" })
    );

    let patched = server.request(
        5,
        "tools/call",
        json!({
            "name": "mcp_submit_support_ticket_edit_patch",
            "arguments": {
                "session_id": session_id,
                "expected_revision": 0,
                "values": {
                    "details": "Captured through an edit session",
                    "urgent": true
                }
            }
        }),
    );
    assert_eq!(patched["result"]["isError"], false);
    assert_eq!(patched["result"]["structuredContent"]["revision"], 1);
    assert_eq!(
        patched["result"]["structuredContent"]["fields"][0]["label"],
        "Title"
    );

    let edit_submitted = server.request(
        6,
        "tools/call",
        json!({
            "name": "mcp_submit_support_ticket_edit_submit",
            "arguments": {
                "session_id": session_id,
                "expected_revision": 1
            }
        }),
    );
    assert_eq!(edit_submitted["result"]["isError"], false);
    assert_eq!(
        edit_submitted["result"]["structuredContent"],
        json!({
            "accepted": true,
            "kind": "support_ticket",
            "title": "Draft ticket",
            "details": "Captured through an edit session",
            "urgent": true
        })
    );

    server.shutdown();
}
