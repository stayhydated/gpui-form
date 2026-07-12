#![cfg(feature = "mcp")]

use serde::Deserialize;

#[derive(Debug, Deserialize, gpui_form::mcp::McpToolInput, PartialEq)]
#[mcp(crate = gpui_form::mcp)]
struct FacadeToolArgs {
    #[mcp(alias = "q")]
    query: String,
}

#[test]
fn facade_mcp_reexports_component_shape_mcp_derives() {
    let schema = <FacadeToolArgs as gpui_form::mcp::McpJsonSchema>::json_schema();

    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["query"]["type"], "string");
    assert_eq!(
        schema["properties"]["query"]["x-mcpAliases"],
        gpui_form::mcp::serde_json::json!(["q"])
    );
    assert_eq!(
        schema["required"],
        gpui_form::mcp::serde_json::json!(["query"])
    );
}

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
