//! Experimental MCP submit integration for generated `gpui-form` value holders.
//!
//! This crate intentionally keeps GPUI out of the submit execution path. It
//! owns form holder/model decoding and validation while delegating shared MCP
//! server and stdio serving mechanics to `component-shape-mcp`.
//! MCP servers retain their shared form registry and editor sessions for the
//! host lifetime; completing a call never requests shutdown.

mod contracts;
mod descriptors;
mod editor;
mod errors;
mod options;
mod prompts;
pub mod registry;
mod resources;
mod schema;
mod server;
mod submit;

pub use serde::Serialize;

pub use component_shape_mcp::{
    ComponentShapeFor, ComponentShapeMetadata, ContentBlock, MCP_PROTOCOL_VERSION,
    MCP_VALIDATION_PARAMS_NONE, McpAny, McpArguments, McpIconTheme, McpInput, McpInputShape,
    McpJsonSchema, McpPrimitiveKind, McpPromptArgument, McpPromptMessage, McpPromptResult,
    McpRange, McpRangeBoundKind, McpResourceContents, McpResourceResult, McpRole, McpSchema,
    McpSchemaFn, McpSchemaProperties, McpServer, McpServerBuilder, McpToolAnnotations,
    McpToolArguments, McpToolCall, McpToolError, McpToolIcon, McpToolInput, McpToolMetadata,
    McpToolRegistry, McpToolValue, McpTypedTool, McpValidationIssue, McpValidationParam,
    McpValidationRule, McpValidationScope, McpValidationTarget, McpValidationTypeArgMode,
    PromptDefinition, ResourceDefinition, ResourceTemplateDefinition, ServeStdioResult,
    ToolCallResult, ToolDefinition, object_schema, serde, serde_json, validation_issues_error,
};
pub use contracts::{
    McpContextSubmit, McpEditableForm, McpForm, McpFormInput, McpFormModel, McpFormValidation,
    McpFormValueHolder, McpObject, McpSubmitArgument, McpSubmitContext,
};
pub use descriptors::{FieldToolValueSchemaFn, McpField, McpFormDescriptor};
pub use editor::{McpFormEditorToolNames, editor_tool_definitions, editor_tool_names};
pub use options::{
    DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT, DEFAULT_EDITOR_SESSION_LIMIT, McpFormEditorOptions,
    McpFormRegistrationOptions, McpFormRegistrationReport, McpSkippedTool,
};
pub use prompts::{
    McpFormPromptNames, form_prompt_names, register_context_submitter_prompt_templates,
    register_form_prompt_templates, register_prompt_templates,
};
pub use resources::{
    McpFormResourceUris, form_descriptor_resource_value, form_examples_resource_value,
    form_resource_uris, form_schema_resource_value, register_form_resources,
};
pub use rmcp;
pub use server::{
    builder, builder_named, register, register_with_options, serve_stdio, serve_stdio_blocking,
    server, server_named, tool_definitions, tool_definitions_with_options, tool_registry,
    tool_registry_with_options,
};
pub use submit::{
    DEFAULT_SERVER_NAME, FormTool, context_submit_tool_definitions, form,
    register_context_submitters, register_context_submitters_strict,
    register_context_submitters_strict_with_editor_options,
    register_context_submitters_with_editor_options, register_editor, register_editor_with_options,
    submit_tool_definition, submit_tool_definition_for_response, tool_name,
};

pub(crate) use editor::*;
pub(crate) use errors::*;
pub(crate) use resources::*;
pub(crate) use schema::*;
pub(crate) use server::*;
pub(crate) use submit::*;

#[cfg(test)]
mod tests;
