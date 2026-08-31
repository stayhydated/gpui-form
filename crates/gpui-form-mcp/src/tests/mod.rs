pub(super) use super::{
    McpField, McpFormDescriptor, McpFormEditorOptions, McpFormInput, McpFormRegistrationOptions,
    McpInput, McpObject, McpServer, form, tool_name,
};
pub(super) use gpui_form_schema::registry::{FieldValuePresence, RustPath, RustType};
pub(super) use std::time::Duration;

pub(super) struct StringComponentShape;

impl super::ComponentShapeMetadata for StringComponentShape {
    const MCP_INPUT: McpInput = McpInput::number();
}
impl super::ComponentShapeFor<String> for StringComponentShape {
    const MCP_INPUT: McpInput = McpInput::number();
}

mod descriptors_schema;
mod editor;
mod registration_server;
mod resources_prompts;

pub(super) struct Booking;

impl super::McpForm for Booking {
    type ValueHolder = BookingHolder;

    fn descriptor() -> McpFormDescriptor {
        const FIELDS: &[McpField] = &[McpField::typed::<String>(
            "available_on",
            RustType::from_macro_tokens_unchecked("String"),
            FieldValuePresence::DirectStorage,
        )
        .with_examples(&["2026-06-10"])];

        McpFormDescriptor::new(
            "Booking",
            RustPath::from_macro_tokens_unchecked("tests"),
            FIELDS,
            false,
            component_shape_mcp::McpToolMetadata::new()
                .with_name("reserve_booking")
                .with_title("Reserve booking")
                .with_description("Reserve a booking window."),
        )
    }

    fn decode_arguments(
        call: super::McpToolCall,
    ) -> Result<Self::ValueHolder, super::McpToolError> {
        let mut arguments = call.into_arguments();
        let available_on = arguments
            .take_present_tool_value::<String>("available_on")?
            .unwrap_or_default();
        arguments.finish()?;
        Ok(BookingHolder { available_on })
    }

    fn validate_holder(_holder: &Self::ValueHolder) -> Result<(), super::McpToolError> {
        Ok(())
    }
}

pub(super) struct BookingHolder {
    pub(super) available_on: String,
}

#[derive(serde::Serialize)]
pub(super) struct BookingResponse {
    pub(super) available_on: String,
}

impl super::McpJsonSchema for BookingResponse {
    fn json_schema() -> super::McpSchema {
        let mut properties = super::McpSchemaProperties::new();
        properties.insert(
            "available_on".to_string(),
            <String as super::McpJsonSchema>::json_schema(),
        );
        super::object_schema(properties, ["available_on"])
    }
}
