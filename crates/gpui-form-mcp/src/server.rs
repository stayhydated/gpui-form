use std::collections::BTreeSet;

use crate::*;

/// Build a server containing every inventory-discovered form submit handler and
/// editable form session tool.
pub fn server() -> Result<McpServer, McpToolError> {
    builder().build()
}

/// Build the inventory-discovered form tool registry.
pub fn tool_registry() -> Result<McpToolRegistry, McpToolError> {
    tool_registry_with_options(McpFormRegistrationOptions::all())
}

/// Build the inventory-discovered form tool registry for a registration profile.
pub fn tool_registry_with_options(
    options: McpFormRegistrationOptions,
) -> Result<McpToolRegistry, McpToolError> {
    let mut server = McpServer::new(DEFAULT_SERVER_NAME, env!("CARGO_PKG_VERSION"));
    register_with_options(&mut server, options)?;
    Ok(server.into_tool_registry())
}

/// Serve every inventory-discovered form submit handler and editable form
/// session tool over stdio.
pub async fn serve_stdio() -> ServeStdioResult {
    server()?.serve_stdio().await
}

/// Serve every inventory-discovered form submit handler and editable form
/// session tool over stdio from a synchronous `main`.
pub fn serve_stdio_blocking() -> ServeStdioResult {
    server()?.serve_stdio_blocking()
}

/// Build a generated form submit server with application-owned metadata.
pub fn server_named(
    server_name: impl Into<std::borrow::Cow<'static, str>>,
    server_version: impl Into<std::borrow::Cow<'static, str>>,
) -> Result<McpServer, McpToolError> {
    builder_named(server_name, server_version).build()
}

/// Build a server builder containing every inventory-discovered form submit
/// handler and editable form session tool.
pub fn builder() -> McpServerBuilder {
    builder_named(DEFAULT_SERVER_NAME, env!("CARGO_PKG_VERSION"))
}

/// Build a generated form submit and editor server builder with
/// application-owned metadata.
pub fn builder_named(
    server_name: impl Into<std::borrow::Cow<'static, str>>,
    server_version: impl Into<std::borrow::Cow<'static, str>>,
) -> McpServerBuilder {
    McpServer::builder(server_name, server_version).register(register)
}

/// Add every inventory-discovered form submit handler and editable form session
/// tool to a shared MCP server.
pub fn register(server: &mut McpServer) -> Result<(), McpToolError> {
    register_with_options(server, McpFormRegistrationOptions::all()).map(|_| ())
}

/// Add inventory-discovered form tools to a shared MCP server using a
/// registration profile.
pub fn register_with_options(
    server: &mut McpServer,
    options: McpFormRegistrationOptions,
) -> Result<McpFormRegistrationReport, McpToolError> {
    let report = registration_report_for_options(options)?;
    if !options.registers_tools() {
        return Ok(report);
    }

    let resource_specs = form_resource_specs_for_options(options)?;
    ensure_tool_definitions_available(server, &report.tool_definitions)?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    let submit_tool_names = submit_handler_tool_names();
    for registration in registry::submit_handler_registrations() {
        if !options.submit_handlers {
            continue;
        }
        if options.editors && options.edit_submit {
            registration.register(server)?;
        } else {
            registration.register_submit(server)?;
        }
    }
    for registration in registry::editor_registrations() {
        if !options.editors {
            continue;
        }
        if options.submit_handlers
            && options.edit_submit
            && submit_tool_names.contains(&registration.descriptor().tool_name())
        {
            continue;
        }
        registration.register(server)?;
    }
    Ok(report)
}

/// Return MCP tool definitions selected by a registration profile without
/// mutating a server.
pub fn tool_definitions_with_options(
    options: McpFormRegistrationOptions,
) -> Result<Vec<ToolDefinition>, McpToolError> {
    Ok(registration_report_for_options(options)?.tool_definitions)
}

pub(crate) fn registration_report_for_options(
    options: McpFormRegistrationOptions,
) -> Result<McpFormRegistrationReport, McpToolError> {
    if !options.registers_tools() {
        return Ok(
            McpFormRegistrationReport::from_tool_definitions(tool_definitions()?).metadata_only(),
        );
    }

    let mut seen = BTreeSet::new();
    let mut definitions = Vec::new();
    let mut skipped_tools = Vec::new();
    let submit_tool_names = submit_handler_tool_names();

    for registration in registry::submit_handler_registrations() {
        let descriptor = registration.descriptor();
        if options.submit_handlers {
            let tool_definitions = if options.editors && options.edit_submit {
                registration.tool_definitions()?
            } else {
                registration.submit_tool_definitions()?
            };
            for definition in tool_definitions {
                push_tool_definition(&mut seen, &mut definitions, definition)?;
            }
        } else {
            push_skipped_tool(
                &mut skipped_tools,
                descriptor.tool_name(),
                "submit_handlers_disabled",
            );
        }

        if !options.editors {
            push_skipped_editor_tool_names(
                &mut skipped_tools,
                descriptor,
                "editors_disabled",
                options.edit_submit,
            );
            if !options.edit_submit {
                push_skipped_tool(
                    &mut skipped_tools,
                    editor_tool_names(descriptor).submit,
                    "edit_submit_disabled",
                );
            }
        } else if !options.edit_submit {
            push_skipped_tool(
                &mut skipped_tools,
                editor_tool_names(descriptor).submit,
                "edit_submit_disabled",
            );
        } else if !options.submit_handlers {
            push_skipped_tool(
                &mut skipped_tools,
                editor_tool_names(descriptor).submit,
                "submit_handlers_disabled",
            );
        }
    }

    for registration in registry::editor_registrations() {
        let descriptor = registration.descriptor();
        let covered_by_submit_handler = options.submit_handlers
            && options.edit_submit
            && submit_tool_names.contains(&descriptor.tool_name());
        if options.editors && !covered_by_submit_handler {
            for definition in registration.tool_definitions()? {
                push_tool_definition(&mut seen, &mut definitions, definition)?;
            }
        } else if !options.editors && !submit_tool_names.contains(&descriptor.tool_name()) {
            push_skipped_editor_tool_names(
                &mut skipped_tools,
                descriptor,
                "editors_disabled",
                false,
            );
        }
    }

    let registered_tool_names = definitions
        .iter()
        .map(|definition| definition.name.to_string())
        .collect();
    Ok(McpFormRegistrationReport {
        tool_definitions: definitions,
        registered_tool_names,
        skipped_tools,
        context_registration_count: 0,
    })
}

pub(crate) fn push_skipped_editor_tool_names(
    skipped_tools: &mut Vec<McpSkippedTool>,
    descriptor: McpFormDescriptor,
    reason: &'static str,
    include_submit: bool,
) {
    let names = editor_tool_names(descriptor);
    for name in [
        names.open,
        names.list,
        names.read,
        names.patch,
        names.validate,
        names.close,
        names.close_all,
    ] {
        push_skipped_tool(skipped_tools, name, reason);
    }
    if include_submit {
        push_skipped_tool(skipped_tools, names.submit, reason);
    }
}

pub(crate) fn push_skipped_tool(
    skipped_tools: &mut Vec<McpSkippedTool>,
    name: String,
    reason: &'static str,
) {
    if skipped_tools.iter().any(|skipped| skipped.name == name) {
        return;
    }
    skipped_tools.push(McpSkippedTool { name, reason });
}

/// Return MCP tool definitions registered by the default generated server,
/// failing on duplicate names.
pub fn tool_definitions() -> Result<Vec<ToolDefinition>, McpToolError> {
    let mut seen = BTreeSet::new();
    let mut tools = Vec::new();
    let submit_tool_names = submit_handler_tool_names();
    for registration in registry::submit_handler_registrations() {
        for definition in registration.tool_definitions()? {
            push_tool_definition(&mut seen, &mut tools, definition)?;
        }
    }
    for registration in registry::editor_registrations() {
        if submit_tool_names.contains(&registration.descriptor().tool_name()) {
            continue;
        }
        for definition in registration.tool_definitions()? {
            push_tool_definition(&mut seen, &mut tools, definition)?;
        }
    }
    Ok(tools)
}

pub(crate) fn submit_handler_tool_names() -> BTreeSet<String> {
    registry::submit_handler_registrations()
        .map(|registration| registration.descriptor().tool_name())
        .collect()
}

pub(crate) fn push_tool_definition(
    seen: &mut BTreeSet<String>,
    tools: &mut Vec<ToolDefinition>,
    definition: ToolDefinition,
) -> Result<(), McpToolError> {
    let name = definition.name.to_string();
    if !seen.insert(name.clone()) {
        return Err(McpToolError::duplicate_tool(name));
    }
    tools.push(definition);
    Ok(())
}
