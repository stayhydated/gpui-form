use super::*;

#[test]
fn async_editor_submit_does_not_close_session_if_revision_changes_before_success() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));

    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model_async_with_editor({
            let release_rx = Arc::clone(&release_rx);
            move |request| {
                let started_tx = started_tx.clone();
                let release_rx = Arc::clone(&release_rx);
                async move {
                    started_tx.send(()).expect("submit should signal start");
                    release_rx
                        .lock()
                        .expect("release receiver should lock")
                        .recv()
                        .expect("submit should be released");
                    Ok::<ObjectResponse, String>(object_response(json!({
                        "title": request.title,
                        "mode": "async-submit"
                    })))
                }
            }
        })
        .expect("async submit and editor tools should register");

    let tools = editor_tool_names(PlainRequest::descriptor());
    let open = server.call_tool(
        &tools.open,
        Some(json!({ "values": { "title": "Initial title" } })),
    );
    assert_eq!(open.is_error, Some(false));
    let opened = open.structured_content.expect("open should return state");
    assert_eq!(opened["revision"], json!(0));
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string")
        .to_string();

    let submit_server = server.clone();
    let submit_tool = tools.submit.clone();
    let submit_session_id = session_id.clone();
    let submit_handle = std::thread::spawn(move || {
        submit_server.call_tool(
            &submit_tool,
            Some(json!({
                "session_id": submit_session_id,
                "expected_revision": 0
            })),
        )
    });

    started_rx.recv().expect("submit should start");

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0,
            "values": {
                "title": "Updated while submit runs"
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("patch should return state");
    assert_eq!(patched["revision"], json!(1));
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Updated while submit runs" })
    );

    release_tx.send(()).expect("submit should release");
    let submit = submit_handle.join().expect("submit thread should join");
    assert_eq!(submit.is_error, Some(false));
    assert_eq!(
        submit.structured_content,
        Some(json!({
            "title": "Initial title",
            "mode": "async-submit"
        }))
    );

    let read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(read.is_error, Some(false));
    let read = read
        .structured_content
        .expect("newer revision should remain open");
    assert_eq!(read["revision"], json!(1));
    assert_eq!(
        read["submit_arguments"],
        json!({ "title": "Updated while submit runs" })
    );
}

#[test]
fn generated_mcp_editor_decodes_single_field_updates() {
    let mut holder = PlainRequestFormValueHolder::default();

    PlainRequest::decode_field(&mut holder, "title", McpAny::from(json!("Draft")))
        .expect("title should patch");
    PlainRequest::decode_field(&mut holder, "retries", McpAny::from(json!(3)))
        .expect("optional value should patch");
    PlainRequest::decode_field(&mut holder, "enabled", McpAny::from(json!(false)))
        .expect("defaulted value should patch");

    assert_eq!(holder.title, "Draft");
    assert_eq!(holder.retries, Some(3));
    assert!(!holder.enabled);

    let error = PlainRequest::decode_field(&mut holder, "unknown", McpAny::from(json!(true)))
        .expect_err("unknown fields should fail");
    assert_eq!(
        error,
        McpToolError::UnknownField {
            field: "unknown".to_string()
        }
    );
}

#[test]
fn editor_tools_open_patch_read_validate_and_close_sessions() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor()
        .expect("editor tools should register");
    let tools = editor_tool_names(PlainRequest::descriptor());
    assert!(!server.contains_tool(&tools.submit));
    let default_idle_timeout_ms = DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT.as_millis() as u64;

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list should return sessions")["session_count"],
        json!(0)
    );
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list should return sessions")["session_limit"],
        json!(DEFAULT_EDITOR_SESSION_LIMIT)
    );
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list should return sessions")["session_idle_timeout_ms"],
        json!(default_idle_timeout_ms)
    );
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list should return sessions")["cleanup"],
        json!({
            "expired_count": 0,
            "expired_session_ids": [],
            "evicted_count": 0,
            "evicted_session_ids": []
        })
    );

    let open = server.call_tool(&tools.open, Some(json!({})));
    assert_eq!(open.is_error, Some(false));
    let opened = open
        .structured_content
        .expect("open should return a session");
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string");
    assert_eq!(opened["revision"], json!(0));
    assert_eq!(
        opened["cleanup"],
        json!({
            "expired_count": 0,
            "expired_session_ids": [],
            "evicted_count": 0,
            "evicted_session_ids": []
        })
    );
    assert_eq!(opened["missing_required"], json!(["title"]));
    assert_eq!(opened["valid"], false);
    assert_eq!(
        opened["errors"],
        json!([{
            "scope": "field",
            "message": "missing required field `title`",
            "field": "title",
            "validator": "required"
        }])
    );
    assert_eq!(opened["fields"][0]["name"], "title");
    assert_eq!(opened["fields"][0]["required"], true);
    assert_eq!(opened["fields"][0]["present"], false);
    assert_eq!(opened["fields"][0]["has_value"], false);
    assert_eq!(opened["fields"][0]["value"], Value::Null);
    assert_eq!(opened["fields"][0]["missing"], true);
    assert_eq!(
        opened["fields"][0]["errors"],
        json!([{
            "scope": "field",
            "message": "missing required field `title`",
            "field": "title",
            "validator": "required"
        }])
    );
    assert_eq!(opened["fields"][0]["schema"]["type"], "string");
    assert_eq!(opened["fields"][2]["name"], "enabled");
    assert_eq!(opened["fields"][2]["required"], false);
    assert_eq!(opened["fields"][2]["has_default"], true);
    assert_eq!(opened["fields"][2]["errors"], json!([]));
    assert_eq!(opened["submit_arguments"], json!({}));

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    let listed = listed
        .structured_content
        .expect("list should return active session");
    assert_eq!(listed["session_count"], json!(1));
    assert_eq!(listed["sessions"][0]["session_id"], session_id);
    assert_eq!(listed["sessions"][0]["revision"], json!(0));
    assert_eq!(listed["sessions"][0]["valid"], false);
    assert_eq!(listed["sessions"][0]["fields"][0]["missing"], true);

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0,
            "values": {
                "title": "Edited title"
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("patch should return state");
    assert_eq!(patched["revision"], json!(1));
    assert_eq!(patched["missing_required"], json!([]));
    assert_eq!(patched["valid"], true);
    assert_eq!(patched["errors"], json!([]));
    assert_eq!(patched["fields"][0]["present"], true);
    assert_eq!(patched["fields"][0]["has_value"], true);
    assert_eq!(patched["fields"][0]["value"], "Edited title");
    assert_eq!(patched["fields"][0]["missing"], false);
    assert_eq!(patched["fields"][0]["errors"], json!([]));
    assert_eq!(patched["values"]["title"], "Edited title");
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Edited title" })
    );

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    let listed = listed
        .structured_content
        .expect("list should return patched session");
    assert_eq!(
        listed["sessions"][0]["submit_arguments"],
        json!({ "title": "Edited title" })
    );
    assert_eq!(listed["sessions"][0]["revision"], json!(1));

    let stale_patch = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0,
            "values": {
                "title": "Stale title"
            }
        })),
    );
    assert_eq!(stale_patch.is_error, Some(true));
    assert_eq!(
        stale_patch.structured_content.expect("structured error")["error"],
        json!({
            "kind": "validation",
            "message": "validation failed: edit session revision mismatch: expected 0, current 1",
            "detail": "edit session revision mismatch: expected 0, current 1"
        })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 1,
            "values": {
                "retries": 5
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("patch should return state");
    assert_eq!(patched["revision"], json!(2));
    assert_eq!(patched["values"]["retries"], 5);
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Edited title", "retries": 5 })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "clear": ["retries"]
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("clear should return state");
    assert_eq!(patched["valid"], true);
    assert_eq!(patched["values"].get("retries"), None);
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Edited title" })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "values": {
                "retries": null
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("patch should return state");
    assert_eq!(patched["values"]["retries"], Value::Null);
    assert_eq!(patched["fields"][1]["name"], "retries");
    assert_eq!(patched["fields"][1]["has_value"], true);
    assert_eq!(patched["fields"][1]["value"], Value::Null);
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Edited title", "retries": null })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "values": {
                "retries": 5
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));

    let validation = server.call_tool(
        &tools.validate,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(validation.is_error, Some(false));
    let validation = validation
        .structured_content
        .expect("validation should return state");
    assert_eq!(validation["valid"], true);
    assert_eq!(validation["errors"], json!([]));
    assert_eq!(
        validation["submit_arguments"],
        json!({ "title": "Edited title", "retries": 5 })
    );

    let read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(read.is_error, Some(false));
    assert_eq!(
        read.structured_content.expect("read")["submit_arguments"],
        json!({ "title": "Edited title", "retries": 5 })
    );

    let replaced = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "replace": true,
            "values": {
                "title": "Replacement title"
            }
        })),
    );
    assert_eq!(replaced.is_error, Some(false));
    let replaced = replaced
        .structured_content
        .expect("replace patch should return state");
    assert_eq!(replaced["valid"], true);
    assert_eq!(
        replaced["submit_arguments"],
        json!({ "title": "Replacement title" })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "values": {
                "title": "Edited title",
                "retries": 5
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "clear": ["title"]
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("required clear should return state");
    assert_eq!(patched["valid"], false);
    assert_eq!(patched["missing_required"], json!(["title"]));
    assert_eq!(
        patched["fields"][0]["errors"],
        json!([{
            "scope": "field",
            "message": "missing required field `title`",
            "field": "title",
            "validator": "required"
        }])
    );
    assert_eq!(patched["submit_arguments"], json!({ "retries": 5 }));
    let close_revision = patched["revision"]
        .as_u64()
        .expect("patched revision should be an integer");

    let stale_close = server.call_tool(
        &tools.close,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0
        })),
    );
    assert_eq!(stale_close.is_error, Some(true));
    assert_eq!(
        stale_close.structured_content.expect("structured error")["error"],
        json!({
            "kind": "validation",
            "message": format!(
                "validation failed: edit session revision mismatch: expected 0, current {close_revision}"
            ),
            "detail": format!(
                "edit session revision mismatch: expected 0, current {close_revision}"
            )
        })
    );

    let close = server.call_tool(
        &tools.close,
        Some(json!({
            "session_id": session_id,
            "expected_revision": close_revision
        })),
    );
    assert_eq!(close.is_error, Some(false));

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list after close should return no sessions")["session_count"],
        json!(0)
    );

    let read_after_close = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(read_after_close.is_error, Some(true));
    assert_eq!(
        read_after_close
            .structured_content
            .expect("structured error")["error"],
        json!({
            "kind": "invalid_field_value",
            "message": format!("unknown value `{session_id}` for field `session_id`"),
            "field": "session_id",
            "value": session_id
        })
    );

    let first = server.call_tool(&tools.open, Some(json!({})));
    assert_eq!(first.is_error, Some(false));
    let first_id = first.structured_content.unwrap()["session_id"]
        .as_str()
        .expect("first reopened session id")
        .to_string();
    let second = server.call_tool(
        &tools.open,
        Some(json!({
            "values": {
                "title": "Second session"
            }
        })),
    );
    assert_eq!(second.is_error, Some(false));
    let second_id = second.structured_content.unwrap()["session_id"]
        .as_str()
        .expect("second reopened session id")
        .to_string();

    let close_all = server.call_tool(&tools.close_all, Some(json!({})));
    assert_eq!(close_all.is_error, Some(false));
    let close_all = close_all
        .structured_content
        .expect("close_all should return closed session ids");
    assert_eq!(
        close_all["form"],
        json!(PlainRequest::descriptor().tool_name())
    );
    assert_eq!(close_all["form_name"], "PlainRequest");
    assert_eq!(close_all["closed_count"], json!(2));
    assert_eq!(close_all["session_ids"], json!([first_id, second_id]));

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list after close_all should return no sessions")["session_count"],
        json!(0)
    );

    let empty_close_all = server.call_tool(&tools.close_all, Some(json!({})));
    assert_eq!(empty_close_all.is_error, Some(false));
    let empty_close_all = empty_close_all
        .structured_content
        .expect("empty close_all should still succeed");
    assert_eq!(empty_close_all["closed_count"], json!(0));
    assert_eq!(empty_close_all["session_ids"], json!([]));
}

#[test]
fn editor_options_evict_oldest_sessions_past_limit() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor_with_options(McpFormEditorOptions::default().with_session_limit(2))
        .expect("editor tools should register with options");
    let tools = editor_tool_names(PlainRequest::descriptor());

    let mut session_ids = Vec::new();
    let mut third_opened = None;
    for title in ["First", "Second", "Third"] {
        let opened = server.call_tool(
            &tools.open,
            Some(json!({
                "values": {
                    "title": title
                }
            })),
        );
        assert_eq!(opened.is_error, Some(false));
        let opened = opened
            .structured_content
            .expect("open should return a session");
        session_ids.push(
            opened["session_id"]
                .as_str()
                .expect("session id should be a string")
                .to_string(),
        );
        if title == "Third" {
            third_opened = Some(opened);
        }
    }
    let third_opened = third_opened.expect("third open should be captured");
    assert_eq!(
        third_opened["cleanup"],
        json!({
            "expired_count": 0,
            "expired_session_ids": [],
            "evicted_count": 1,
            "evicted_session_ids": [session_ids[0]]
        })
    );

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    let listed = listed
        .structured_content
        .expect("list should return bounded sessions");
    assert_eq!(listed["session_limit"], json!(2));
    assert_eq!(
        listed["cleanup"],
        json!({
            "expired_count": 0,
            "expired_session_ids": [],
            "evicted_count": 0,
            "evicted_session_ids": []
        })
    );
    assert_eq!(listed["session_count"], json!(2));
    assert_eq!(listed["sessions"][0]["session_id"], session_ids[1]);
    assert_eq!(
        listed["sessions"][0]["submit_arguments"],
        json!({ "title": "Second" })
    );
    assert_eq!(listed["sessions"][1]["session_id"], session_ids[2]);
    assert_eq!(
        listed["sessions"][1]["submit_arguments"],
        json!({ "title": "Third" })
    );

    let evicted_read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_ids[0]
        })),
    );
    assert_eq!(evicted_read.is_error, Some(true));
    assert_eq!(
        evicted_read
            .structured_content
            .expect("read should return a structured error")["error"]["kind"],
        json!("invalid_field_value")
    );
}

#[test]
fn editor_options_expire_idle_sessions_before_next_operation() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor_with_options(
            McpFormEditorOptions::default().with_session_idle_timeout(Duration::ZERO),
        )
        .expect("editor tools should register with idle timeout");
    let tools = editor_tool_names(PlainRequest::descriptor());

    let opened = server.call_tool(
        &tools.open,
        Some(json!({
            "values": {
                "title": "Expires immediately after open"
            }
        })),
    );
    assert_eq!(opened.is_error, Some(false));
    let session_id = opened
        .structured_content
        .expect("open should return a session")["session_id"]
        .as_str()
        .expect("session id should be a string")
        .to_string();

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    let listed = listed
        .structured_content
        .expect("list should expire idle sessions");
    assert_eq!(listed["session_idle_timeout_ms"], json!(0));
    assert_eq!(
        listed["cleanup"],
        json!({
            "expired_count": 1,
            "expired_session_ids": [session_id],
            "evicted_count": 0,
            "evicted_session_ids": []
        })
    );
    assert_eq!(listed["session_count"], json!(0));

    let expired_read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(expired_read.is_error, Some(true));
    assert_eq!(
        expired_read
            .structured_content
            .expect("read should return a structured error")["error"]["kind"],
        json!("invalid_field_value")
    );
}

#[test]
fn editor_bulk_patch_rejects_invalid_values_without_partial_mutation() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor()
        .expect("editor tools should register");
    let tools = editor_tool_names(PlainRequest::descriptor());

    let open = server.call_tool(&tools.open, Some(json!({})));
    assert_eq!(open.is_error, Some(false));
    let opened = open
        .structured_content
        .expect("open should return a session");
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string");

    let failed = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "values": {
                "title": "Should not commit",
                "unknown": true
            }
        })),
    );
    assert_eq!(failed.is_error, Some(true));
    assert_eq!(
        failed.structured_content.expect("structured error")["error"],
        json!({
            "kind": "unknown_field",
            "message": "unknown field `unknown`",
            "field": "unknown"
        })
    );

    let read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(read.is_error, Some(false));
    let read = read.structured_content.expect("read should return state");
    assert_eq!(read["submit_arguments"], json!({}));
    assert_eq!(read["missing_required"], json!(["title"]));
    assert_eq!(read["valid"], false);

    let failed = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "clear": ["unknown"]
        })),
    );
    assert_eq!(failed.is_error, Some(true));
    assert_eq!(
        failed.structured_content.expect("structured error")["error"],
        json!({
            "kind": "unknown_field",
            "message": "unknown field `unknown`",
            "field": "unknown"
        })
    );
}
