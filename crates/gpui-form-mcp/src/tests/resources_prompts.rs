use super::*;
use crate::McpForm as _;
use crate::prompts::{McpFormPromptKind, form_prompt_text};

#[test]
fn form_resources_describe_generated_form() {
    let descriptor = Booking::descriptor();
    let resource_uris = crate::form_resource_uris(descriptor);

    assert_eq!(
        resource_uris.descriptor,
        "gpui-form://forms/reserve_booking/descriptor"
    );
    assert_eq!(
        resource_uris.schema,
        "gpui-form://forms/reserve_booking/schema"
    );
    assert_eq!(
        resource_uris.examples,
        "gpui-form://forms/reserve_booking/examples"
    );

    let descriptor_value = crate::form_descriptor_resource_value(descriptor);
    assert_eq!(descriptor_value["form_name"], "Booking");
    assert_eq!(descriptor_value["tool_name"], "reserve_booking");
    assert_eq!(
        descriptor_value["resources"]["schema"],
        "gpui-form://forms/reserve_booking/schema"
    );
    assert_eq!(descriptor_value["fields"][0]["name"], "available_on");
    assert_eq!(descriptor_value["fields"][0]["label"], "Available on");
    assert_eq!(
        descriptor_value["fields"][0]["examples"],
        serde_json::json!(["2026-06-10"])
    );
    assert_eq!(
        descriptor_value["fields"][0]["schema"]["examples"],
        serde_json::json!(["2026-06-10"])
    );

    let schema_value = crate::form_schema_resource_value(descriptor);
    assert_eq!(schema_value["properties"]["available_on"]["type"], "string");

    let examples_value = crate::form_examples_resource_value(descriptor);
    assert_eq!(
        examples_value["examples"]["available_on"],
        serde_json::json!(["2026-06-10"])
    );
}

#[test]
fn form_prompt_templates_describe_generated_form_workflows() {
    let descriptor = Booking::descriptor();
    let prompt_names = crate::form_prompt_names(descriptor);

    assert_eq!(prompt_names.fill, "fill_reserve_booking_form");
    assert_eq!(prompt_names.repair, "repair_reserve_booking_form");
    assert_eq!(prompt_names.submit, "submit_reserve_booking_form");
    assert_eq!(
        prompt_names.all(),
        [
            "fill_reserve_booking_form",
            "repair_reserve_booking_form",
            "submit_reserve_booking_form",
        ]
    );

    let mut server = McpServer::new("example-form-server".to_string(), "9.9.9");
    crate::register_form_prompt_templates::<Booking>(&mut server)
        .expect("prompt templates should register");

    assert_eq!(server.resource_count(), 3);
    assert_eq!(server.prompt_count(), 3);
    for prompt_name in prompt_names.all() {
        assert!(server.contains_prompt(prompt_name));
    }

    let prompts = server.list_prompts();
    let fill = prompts
        .iter()
        .find(|prompt| prompt.name == "fill_reserve_booking_form")
        .expect("fill prompt should be listed");
    assert_eq!(fill.title.as_deref(), Some("Fill Reserve booking"));
    assert_eq!(
        fill.description.as_deref(),
        Some("Draft valid direct-submit arguments for Reserve booking.")
    );
    assert_eq!(
        fill.arguments
            .as_ref()
            .expect("fill prompt should expose arguments")[0]
            .name,
        "goal"
    );

    let repair_text = form_prompt_text(
        descriptor,
        McpFormPromptKind::Repair,
        Some(serde_json::Map::from_iter([(
            "validation_issues".to_string(),
            serde_json::json!([{ "field": "available_on", "message": "required" }]),
        )])),
    );
    assert!(repair_text.contains("gpui-form://forms/reserve_booking/descriptor"));
    assert!(repair_text.contains("reserve_booking_edit_patch"));
    assert!(repair_text.contains("Caller-provided context"));
}

#[test]
fn prompt_first_registration_reuses_form_resources_for_tools() {
    let mut server = McpServer::new("example-form-server".to_string(), "9.9.9");

    crate::register_form_prompt_templates::<Booking>(&mut server)
        .expect("prompt templates should register");
    form::<Booking>(&mut server)
        .holder(|holder| {
            Ok::<_, String>(BookingResponse {
                available_on: holder.available_on,
            })
        })
        .expect("tool should register after prompt resources");

    assert_eq!(server.resource_count(), 3);
    assert_eq!(server.prompt_count(), 3);
    assert!(server.contains_tool("reserve_booking"));
}
