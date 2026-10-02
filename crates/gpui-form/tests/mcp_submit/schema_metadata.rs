use super::*;

#[test]
fn generated_mcp_schema_marks_required_optional_and_default_fields() {
    let schema = PlainRequest::descriptor().input_schema();

    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["title"]["type"], "string");
    assert_eq!(
        schema["properties"]["retries"]["anyOf"][0]["type"],
        "integer"
    );
    assert_eq!(schema["properties"]["enabled"]["type"], "boolean");
    assert_eq!(
        schema["properties"]["enabled"]["x-gpuiFormHasDefault"],
        true
    );
    assert_eq!(schema["required"], json!(["title"]));

    let custom_schema = CustomSchemaRequest::descriptor().input_schema();
    assert_eq!(custom_schema["properties"]["account_id"]["type"], "integer");
    assert_eq!(custom_schema["properties"]["attempts"]["type"], "integer");
    assert_eq!(custom_schema["required"], json!(["account_id", "attempts"]));

    let tool_input_schema = ToolInputSchemaRequest::descriptor().input_schema();
    assert_eq!(
        tool_input_schema["properties"]["preferences"]["type"],
        "object"
    );
    assert_eq!(
        tool_input_schema["properties"]["preferences"]["properties"]["emailUpdates"]["type"],
        "boolean"
    );
    assert_eq!(
        tool_input_schema["properties"]["preferences"]["properties"]["emailUpdates"]["x-mcpAliases"],
        json!(["email"])
    );

    let tuple_schema = TupleSchemaRequest::descriptor().input_schema();
    assert_eq!(
        tuple_schema["properties"]["coordinates"]["prefixItems"][0]["type"],
        "integer"
    );
    assert_eq!(
        tuple_schema["properties"]["coordinates"]["prefixItems"][1]["type"],
        "integer"
    );
    assert_eq!(tuple_schema["properties"]["coordinates"]["minItems"], 2);

    let enum_schema = EnumSchemaRequest::descriptor().input_schema();
    assert_eq!(enum_schema["properties"]["priority"]["type"], "string");
    assert_eq!(
        enum_schema["properties"]["priority"]["enum"],
        json!(["low", "high-priority", "urgent"])
    );
}

#[cfg(feature = "runtime")]
#[test]
fn generated_mcp_schema_marks_component_backed_fields() {
    let component_schema = ComponentRequest::descriptor().input_schema();
    assert_eq!(
        component_schema["properties"]["title"]["x-gpuiFormComponentBacked"],
        true
    );
}

#[test]
fn generated_mcp_schema_publishes_koruma_validation_metadata_and_constraints() {
    let non_empty_schema = ValidatedRequest::descriptor().input_schema();
    let name_schema = &non_empty_schema["properties"]["name"];
    assert_eq!(name_schema["minLength"], json!(1));
    assert_eq!(
        name_schema["x-gpuiFormValidation"],
        json!([{
            "scope": "field",
            "validator": "NonEmptyValidation",
            "path": "NonEmptyValidation",
            "target": "default",
            "type_arg_mode": "infer"
        }])
    );

    let constrained_schema = ConstrainedRequest::descriptor().input_schema();
    let title_schema = &constrained_schema["properties"]["title"];
    assert_eq!(title_schema["minLength"], json!(2));
    assert_eq!(title_schema["maxLength"], json!(8));
    assert_eq!(
        title_schema["x-gpuiFormValidation"][0]["params"],
        json!([
            { "name": "min", "value": "2" },
            { "name": "max", "value": "8" }
        ])
    );

    let retries_schema = &constrained_schema["properties"]["retries"];
    assert_eq!(retries_schema["minimum"], json!(1));
    assert_eq!(retries_schema["exclusiveMaximum"], json!(5));
    assert_eq!(
        retries_schema["x-gpuiFormValidation"][0]["params"],
        json!([
            { "name": "min", "value": "1" },
            { "name": "max", "value": "5" },
            { "name": "exclusive_max", "value": "true" }
        ])
    );
}

#[test]
fn generated_mcp_descriptor_uses_explicit_metadata() {
    let descriptor = MetadataRequest::descriptor();

    assert_eq!(descriptor.tool_name(), "submit_metadata_request");
    assert_eq!(descriptor.title(), "Submit metadata request");
    assert_eq!(
        descriptor.description(),
        "Exercise explicit form MCP metadata."
    );
    let annotations = descriptor
        .tool_annotations()
        .expect("explicit mcp hints should publish annotations");
    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, Some(false));
    assert_eq!(annotations.idempotent_hint, Some(true));
    assert_eq!(annotations.open_world_hint, Some(false));
    let icons = descriptor
        .tool_icons()
        .expect("explicit icon metadata should publish icons");
    assert_eq!(icons[0].src, "https://example.com/submit-metadata.png");
    assert_eq!(icons[0].mime_type.as_deref(), Some("image/png"));
}

#[test]
fn generated_mcp_descriptor_infers_doc_description() {
    assert_eq!(
        PlainRequest::descriptor().description(),
        "Submit a plain request from inferred docs."
    );
}

#[test]
fn generated_mcp_fields_publish_inferred_and_explicit_metadata() {
    let descriptor = PlainRequest::descriptor();
    let fields = descriptor.fields();
    let title = fields
        .iter()
        .find(|field| field.name() == "title")
        .expect("title field should be described");
    let retries = fields
        .iter()
        .find(|field| field.name() == "retries")
        .expect("retries field should be described");

    assert_eq!(title.explicit_label(), Some("Support title"));
    assert_eq!(title.label(), "Support title");
    assert_eq!(
        title.description(),
        Some("Short title shown to agents and users.")
    );
    assert_eq!(title.examples(), &["Cannot log in"]);
    assert_eq!(retries.explicit_label(), None);
    assert_eq!(retries.label(), "Retries");
    assert_eq!(
        retries.description(),
        Some("Number of retry attempts to allow.")
    );

    let schema = descriptor.input_schema();
    assert_eq!(schema["properties"]["title"]["title"], "Support title");
    assert_eq!(
        schema["properties"]["title"]["description"],
        "Short title shown to agents and users."
    );
    assert_eq!(
        schema["properties"]["title"]["x-gpuiFormExplicitLabel"],
        "Support title"
    );
    assert_eq!(
        schema["properties"]["title"]["examples"],
        json!(["Cannot log in"])
    );
    assert_eq!(schema["properties"]["retries"]["title"], "Retries");
    assert_eq!(
        schema["properties"]["retries"]["description"],
        "Number of retry attempts to allow."
    );

    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor()
        .expect("editor tools should register");
    let tools = editor_tool_names(descriptor);
    let opened = server.call_tool(&tools.open, Some(json!({})));
    assert_eq!(opened.is_error, Some(false));
    let opened = opened
        .structured_content
        .expect("open should return a session");
    assert_eq!(opened["fields"][0]["label"], "Support title");
    assert_eq!(
        opened["fields"][0]["description"],
        "Short title shown to agents and users."
    );
    assert_eq!(opened["fields"][0]["examples"], json!(["Cannot log in"]));
    assert_eq!(opened["fields"][1]["label"], "Retries");
    assert_eq!(
        opened["fields"][1]["description"],
        "Number of retry attempts to allow."
    );
}

#[test]
fn editor_output_schemas_preserve_invalid_drafts_without_relaxing_input_constraints() {
    let mut server = test_server();
    form::<ConstrainedRequest>(&mut server)
        .editor()
        .expect("editor tools should register");
    let names = editor_tool_names(ConstrainedRequest::descriptor());
    let draft = json!({ "title": "", "retries": 0 });
    let opened = server.call_tool(&names.open, Some(json!({ "values": draft })));
    assert_eq!(opened.is_error, Some(false));
    let opened = opened
        .structured_content
        .expect("invalid draft should remain editable");
    assert_eq!(opened["valid"], false);
    assert_eq!(opened["values"], draft);
    assert_eq!(opened["submit_arguments"], draft);
    assert!(!opened["errors"].as_array().unwrap().is_empty());

    let definitions = server.list_tools();
    for name in [
        &names.open,
        &names.read,
        &names.patch,
        &names.validate,
        &names.list,
    ] {
        let tool = definitions
            .iter()
            .find(|tool| tool.name.as_ref() == name)
            .unwrap();
        let output = Value::Object(tool.output_schema.as_ref().unwrap().as_ref().clone());
        let snapshot = if name == &names.list {
            &output["properties"]["sessions"]["items"]
        } else {
            &output
        };
        assert_draft_snapshot_schema(snapshot, &opened);
    }
    for name in [&names.open, &names.patch] {
        let tools = server.list_tools();
        let tool = tools
            .iter()
            .find(|tool| tool.name.as_ref() == name)
            .unwrap();
        assert_eq!(
            tool.input_schema["properties"]["values"]["properties"]["title"]["minLength"],
            2
        );
        assert_eq!(
            tool.input_schema["properties"]["values"]["properties"]["retries"]["minimum"],
            1
        );
    }
}

fn assert_draft_snapshot_schema(schema: &Value, opened: &Value) {
    for key in ["values", "submit_arguments"] {
        let values = &schema["properties"][key];
        assert_eq!(values["additionalProperties"], false);
        assert_eq!(values["properties"]["title"]["type"], "string");
        assert!(values["properties"]["title"].get("minLength").is_none());
        assert!(values["properties"]["title"].get("maxLength").is_none());
        assert_eq!(values["properties"]["retries"]["type"], "integer");
        assert!(
            values["properties"]["retries"]
                .get("exclusiveMaximum")
                .is_none()
        );
        assert_ne!(values["properties"]["retries"]["minimum"], json!(1));
    }
    let fields = schema["properties"]["fields"]["items"]["oneOf"]
        .as_array()
        .unwrap();
    for (index, field) in fields.iter().enumerate() {
        let properties = &field["properties"];
        assert_eq!(
            properties["schema"]["const"],
            opened["fields"][index]["schema"]
        );
        let value = &properties["value"]["anyOf"][0];
        assert!(value.get("minLength").is_none());
        assert!(value.get("exclusiveMaximum").is_none());
    }
    assert_eq!(opened["fields"][0]["schema"]["minLength"], 2);
    assert_eq!(opened["fields"][1]["schema"]["minimum"], 1);
}
