use super::*;
use crate::McpJsonSchema as _;

#[test]
fn tool_names_are_stable_and_mcp_friendly() {
    assert_eq!(
        tool_name("example::forms", "ContactRequest"),
        "example_forms_contact_request"
    );
    assert_eq!(tool_name("123", ""), "form_123");
}

#[test]
fn dynamic_object_responses_round_trip_json_objects() {
    let object = serde_json::Map::from_iter([("ok".to_owned(), serde_json::json!(true))]);
    let response = McpObject::new(object.clone());
    assert_eq!(response.as_map(), &object);
    assert_eq!(response.clone().into_map(), object);
    assert_eq!(response.into_value(), serde_json::json!({ "ok": true }));

    let converted = McpObject::try_from(serde_json::json!({ "value": 7 })).unwrap();
    let value: serde_json::Value = converted.into();
    assert_eq!(value, serde_json::json!({ "value": 7 }));
    assert!(McpObject::try_from(serde_json::json!([1, 2])).is_err());
    assert_eq!(McpObject::empty(), McpObject::default());
    assert_eq!(McpObject::json_schema()["type"], "object");
}

#[test]
fn fields_publish_defaults_labels_and_requiredness() {
    let field = McpField::typed::<String>(
        "display_name",
        RustType::from_macro_tokens_unchecked("String"),
        FieldValuePresence::DirectStorage,
    )
    .with_label("Public name")
    .with_description("Name shown to other users")
    .with_examples(&["Ada"])
    .with_default(true);

    assert_eq!(field.name(), "display_name");
    assert_eq!(field.value_type().as_str(), "String");
    assert_eq!(field.presence(), FieldValuePresence::DirectStorage);
    assert!(field.has_default());
    assert!(!field.component_backed());
    assert!(!field.mcp_input().supported());
    assert_eq!(field.explicit_label(), Some("Public name"));
    assert_eq!(field.label(), "Public name");
    assert_eq!(field.description(), Some("Name shown to other users"));
    assert_eq!(field.examples(), &["Ada"]);
    assert!(field.validation_rules().is_empty());
    assert!(!field.required());
    assert_eq!((field.tool_value_schema())()["type"], "string");

    let inferred = McpField::typed::<String>(
        "billing_address-line",
        RustType::from_macro_tokens_unchecked("String"),
        FieldValuePresence::RequiresValue,
    );
    assert_eq!(inferred.label(), "Billing address line");
    assert!(inferred.required());
}

#[test]
fn input_schema_marks_required_and_optional_fields() {
    const FIELDS: &[McpField] = &[
        McpField::typed::<String>(
            "email",
            RustType::from_macro_tokens_unchecked("String"),
            FieldValuePresence::DirectStorage,
        ),
        McpField::typed::<u32>(
            "age",
            RustType::from_macro_tokens_unchecked("u32"),
            FieldValuePresence::Optional,
        ),
    ];
    let schema = McpFormDescriptor::new(
        "Contact",
        RustPath::from_macro_tokens_unchecked("tests"),
        FIELDS,
        false,
        component_shape_mcp::McpToolMetadata::new(),
    )
    .input_schema();

    assert_eq!(schema["properties"]["email"]["type"], "string");
    assert_eq!(schema["properties"]["age"]["anyOf"][0]["type"], "integer");
    assert_eq!(schema["required"], serde_json::json!(["email"]));
}

#[test]
fn input_schema_records_component_mcp_input_when_present() {
    const FIELDS: &[McpField] = &[McpField::component::<String, StringComponentShape>(
        "available_on",
        RustType::from_macro_tokens_unchecked("String"),
        FieldValuePresence::DirectStorage,
    )];
    let schema = McpFormDescriptor::new(
        "Booking",
        RustPath::from_macro_tokens_unchecked("tests"),
        FIELDS,
        false,
        component_shape_mcp::McpToolMetadata::new(),
    )
    .input_schema();

    assert_eq!(schema["properties"]["available_on"]["type"], "string");
    assert_eq!(
        schema["properties"]["available_on"]["x-gpuiFormComponentMcpInput"]["type"],
        "number"
    );
    assert_eq!(
        schema["properties"]["available_on"]["x-gpuiFormComponentBacked"],
        true
    );
}

#[test]
fn input_schema_supports_optional_wrapped_rust_type() {
    const FIELDS: &[McpField] = &[McpField::typed::<Option<String>>(
        "notes",
        RustType::from_macro_tokens_unchecked("Option<String>"),
        FieldValuePresence::DirectStorage,
    )];
    let schema = McpFormDescriptor::new(
        "Request",
        RustPath::from_macro_tokens_unchecked("tests"),
        FIELDS,
        false,
        component_shape_mcp::McpToolMetadata::new(),
    )
    .input_schema();

    assert_eq!(schema["properties"]["notes"]["anyOf"][0]["type"], "string");
    assert_eq!(schema["properties"]["notes"]["anyOf"][1]["type"], "null");
}

#[test]
fn input_schema_supports_nested_optional_vec_types() {
    const FIELDS: &[McpField] = &[McpField::typed::<Vec<Option<String>>>(
        "tags",
        RustType::from_macro_tokens_unchecked("Vec<Option<String>>"),
        FieldValuePresence::DirectStorage,
    )];
    let schema = McpFormDescriptor::new(
        "Request",
        RustPath::from_macro_tokens_unchecked("tests"),
        FIELDS,
        false,
        component_shape_mcp::McpToolMetadata::new(),
    )
    .input_schema();

    assert_eq!(schema["properties"]["tags"]["type"], "array");
    assert_eq!(
        schema["properties"]["tags"]["items"]["anyOf"][0]["type"],
        "string"
    );
    assert_eq!(
        schema["properties"]["tags"]["items"]["anyOf"][1]["type"],
        "null"
    );
}

#[cfg(feature = "chrono")]
#[test]
fn mcp_json_schema_supports_chrono_when_feature_enabled() {
    let schema = <chrono::NaiveDate as crate::McpJsonSchema>::json_schema();

    assert_eq!(schema["type"], "string");
    assert_eq!(schema["format"], "date");
}

#[cfg(feature = "rust_decimal")]
#[test]
fn mcp_json_schema_supports_decimal_when_feature_enabled() {
    let schema = <rust_decimal::Decimal as crate::McpJsonSchema>::json_schema();

    assert_eq!(schema["anyOf"][0]["type"], "number");
    assert_eq!(schema["anyOf"][1]["type"], "string");
}

#[test]
fn input_schema_uses_field_tool_value_schema_when_no_component_input_exists() {
    type UserId = u64;

    const FIELDS: &[McpField] = &[McpField::typed::<UserId>(
        "user_id",
        RustType::from_macro_tokens_unchecked("UserId"),
        FieldValuePresence::DirectStorage,
    )];

    let schema = McpFormDescriptor::new(
        "Request",
        RustPath::from_macro_tokens_unchecked("tests"),
        FIELDS,
        false,
        component_shape_mcp::McpToolMetadata::new(),
    )
    .input_schema();

    assert_eq!(schema["properties"]["user_id"]["type"], "integer");
    assert_eq!(schema["properties"]["user_id"]["x-rustType"], "UserId");
}

#[test]
fn input_schema_keeps_field_tool_value_schema_ahead_of_component_mcp_input() {
    const FIELDS: &[McpField] = &[McpField::component::<String, StringComponentShape>(
        "window",
        RustType::from_macro_tokens_unchecked("String"),
        FieldValuePresence::DirectStorage,
    )];

    let schema = McpFormDescriptor::new(
        "Request",
        RustPath::from_macro_tokens_unchecked("tests"),
        FIELDS,
        false,
        component_shape_mcp::McpToolMetadata::new(),
    )
    .input_schema();

    assert_eq!(schema["properties"]["window"]["type"], "string");
    assert_eq!(
        schema["properties"]["window"]["x-gpuiFormComponentMcpInput"]["type"],
        "number"
    );
    assert_eq!(schema["properties"]["window"]["x-rustType"], "String");
}
