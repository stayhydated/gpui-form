#![cfg(feature = "mcp")]

#[cfg(feature = "chrono")]
#[test]
fn facade_mcp_schema_supports_chrono_when_feature_enabled() {
    let schema = <chrono::NaiveDate as gpui_form::mcp::McpJsonSchema>::json_schema();

    assert_eq!(schema["type"], "string");
    assert_eq!(schema["format"], "date");
}

#[cfg(feature = "rust_decimal")]
#[test]
fn facade_mcp_schema_supports_decimal_when_feature_enabled() {
    let schema = <rust_decimal::Decimal as gpui_form::mcp::McpJsonSchema>::json_schema();

    assert_eq!(schema["anyOf"][0]["type"], "number");
    assert_eq!(schema["anyOf"][1]["type"], "string");
}
