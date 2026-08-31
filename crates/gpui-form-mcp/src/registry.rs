use std::any::TypeId;

use super::{
    McpFormDescriptor, McpFormEditorOptions, McpServer, McpSubmitContext, McpToolError,
    ToolDefinition,
};
pub use inventory;

inventory::collect!(McpEditorRegistration);
inventory::collect!(McpSubmitRegistration);
inventory::collect!(McpSubmitHandlerRegistration);
inventory::collect!(McpContextSubmitRegistration);

pub struct McpSubmitRegistration {
    descriptor: fn() -> McpFormDescriptor,
}

impl McpSubmitRegistration {
    pub const fn new(descriptor: fn() -> McpFormDescriptor) -> Self {
        Self { descriptor }
    }

    pub fn descriptor(&self) -> McpFormDescriptor {
        (self.descriptor)()
    }
}

pub fn submit_registrations() -> impl Iterator<Item = &'static McpSubmitRegistration> {
    inventory::iter::<McpSubmitRegistration>.into_iter()
}

pub struct McpEditorRegistration {
    descriptor: fn() -> McpFormDescriptor,
    register: fn(&mut McpServer) -> Result<(), McpToolError>,
    tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
}

impl McpEditorRegistration {
    pub const fn new(
        descriptor: fn() -> McpFormDescriptor,
        register: fn(&mut McpServer) -> Result<(), McpToolError>,
        tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
    ) -> Self {
        Self {
            descriptor,
            register,
            tool_definitions,
        }
    }

    pub fn descriptor(&self) -> McpFormDescriptor {
        (self.descriptor)()
    }

    pub fn register(&self, server: &mut McpServer) -> Result<(), McpToolError> {
        (self.register)(server)
    }

    pub fn tool_definitions(&self) -> Result<Vec<ToolDefinition>, McpToolError> {
        (self.tool_definitions)()
    }
}

pub fn editor_registrations() -> impl Iterator<Item = &'static McpEditorRegistration> {
    inventory::iter::<McpEditorRegistration>.into_iter()
}

pub struct McpSubmitHandlerRegistration {
    descriptor: fn() -> McpFormDescriptor,
    register: fn(&mut McpServer) -> Result<(), McpToolError>,
    register_submit: fn(&mut McpServer) -> Result<(), McpToolError>,
    tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
    submit_tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
}

impl McpSubmitHandlerRegistration {
    pub const fn new(
        descriptor: fn() -> McpFormDescriptor,
        register: fn(&mut McpServer) -> Result<(), McpToolError>,
        tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
    ) -> Self {
        Self {
            descriptor,
            register,
            register_submit: register,
            tool_definitions,
            submit_tool_definitions: tool_definitions,
        }
    }

    pub const fn new_with_submit_only(
        descriptor: fn() -> McpFormDescriptor,
        register: fn(&mut McpServer) -> Result<(), McpToolError>,
        register_submit: fn(&mut McpServer) -> Result<(), McpToolError>,
        tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
        submit_tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
    ) -> Self {
        Self {
            descriptor,
            register,
            register_submit,
            tool_definitions,
            submit_tool_definitions,
        }
    }

    pub fn descriptor(&self) -> McpFormDescriptor {
        (self.descriptor)()
    }

    pub fn register(&self, server: &mut McpServer) -> Result<(), McpToolError> {
        (self.register)(server)
    }

    pub fn register_submit(&self, server: &mut McpServer) -> Result<(), McpToolError> {
        (self.register_submit)(server)
    }

    pub fn tool_definitions(&self) -> Result<Vec<ToolDefinition>, McpToolError> {
        (self.tool_definitions)()
    }

    pub fn submit_tool_definitions(&self) -> Result<Vec<ToolDefinition>, McpToolError> {
        (self.submit_tool_definitions)()
    }
}

pub fn submit_handler_registrations() -> impl Iterator<Item = &'static McpSubmitHandlerRegistration>
{
    inventory::iter::<McpSubmitHandlerRegistration>.into_iter()
}

pub struct McpContextSubmitRegistration {
    descriptor: fn() -> McpFormDescriptor,
    context_type_id: fn() -> TypeId,
    register:
        fn(&mut McpServer, McpSubmitContext, McpFormEditorOptions) -> Result<(), McpToolError>,
    tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
}

impl McpContextSubmitRegistration {
    pub const fn new(
        descriptor: fn() -> McpFormDescriptor,
        context_type_id: fn() -> TypeId,
        register: fn(
            &mut McpServer,
            McpSubmitContext,
            McpFormEditorOptions,
        ) -> Result<(), McpToolError>,
        tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
    ) -> Self {
        Self {
            descriptor,
            context_type_id,
            register,
            tool_definitions,
        }
    }

    pub fn descriptor(&self) -> McpFormDescriptor {
        (self.descriptor)()
    }

    pub fn context_type_id(&self) -> TypeId {
        (self.context_type_id)()
    }

    pub fn register(
        &self,
        server: &mut McpServer,
        context: McpSubmitContext,
        options: McpFormEditorOptions,
    ) -> Result<(), McpToolError> {
        (self.register)(server, context, options)
    }

    pub fn tool_definitions(&self) -> Result<Vec<ToolDefinition>, McpToolError> {
        (self.tool_definitions)()
    }
}

pub fn context_submit_registrations() -> impl Iterator<Item = &'static McpContextSubmitRegistration>
{
    inventory::iter::<McpContextSubmitRegistration>.into_iter()
}
