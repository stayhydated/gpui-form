use crate::*;

pub(crate) fn ensure_tool_definitions_available(
    server: &McpServer,
    definitions: &[ToolDefinition],
) -> Result<(), McpToolError> {
    for definition in definitions {
        let name = definition.name.to_string();
        if server.contains_tool(&name) {
            return Err(McpToolError::duplicate_tool(name));
        }
    }

    Ok(())
}

pub(crate) fn revision_mismatch_error(
    expected_revision: u64,
    current_revision: u64,
) -> McpToolError {
    McpToolError::validation(format!(
        "edit session revision mismatch: expected {expected_revision}, current {current_revision}"
    ))
}
