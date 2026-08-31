use std::time::Duration;

use crate::*;

pub const DEFAULT_EDITOR_SESSION_LIMIT: usize = 128;
pub const DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Generated MCP tool registration profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpFormRegistrationOptions {
    pub submit_handlers: bool,
    pub editors: bool,
    pub edit_submit: bool,
}

impl McpFormRegistrationOptions {
    /// Register submit handlers, standalone editors, and handler-backed
    /// edit-submit tools.
    pub const fn all() -> Self {
        Self {
            submit_handlers: true,
            editors: true,
            edit_submit: true,
        }
    }

    /// Register only `#[gpui_form::mcp_submit]` submit tools.
    pub const fn submit_only() -> Self {
        Self {
            submit_handlers: true,
            editors: false,
            edit_submit: false,
        }
    }

    /// Register only editable holder-session tools.
    pub const fn editor_only() -> Self {
        Self {
            submit_handlers: false,
            editors: true,
            edit_submit: false,
        }
    }

    /// Build registration metadata without mutating the server.
    pub const fn metadata_only() -> Self {
        Self {
            submit_handlers: false,
            editors: false,
            edit_submit: false,
        }
    }

    pub(crate) const fn registers_tools(self) -> bool {
        self.submit_handlers || self.editors || self.edit_submit
    }
}

impl Default for McpFormRegistrationOptions {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpSkippedTool {
    pub name: String,
    pub reason: &'static str,
}

#[derive(Clone, Debug, Default)]
pub struct McpFormRegistrationReport {
    pub tool_definitions: Vec<ToolDefinition>,
    pub registered_tool_names: Vec<String>,
    pub skipped_tools: Vec<McpSkippedTool>,
    pub context_registration_count: usize,
}

impl McpFormRegistrationReport {
    pub(crate) fn from_tool_definitions(tool_definitions: Vec<ToolDefinition>) -> Self {
        let registered_tool_names = tool_definitions
            .iter()
            .map(|definition| definition.name.to_string())
            .collect();
        Self {
            tool_definitions,
            registered_tool_names,
            skipped_tools: Vec::new(),
            context_registration_count: 0,
        }
    }

    pub(crate) fn metadata_only(mut self) -> Self {
        self.skipped_tools.extend(
            self.registered_tool_names
                .iter()
                .map(|name| McpSkippedTool {
                    name: name.clone(),
                    reason: "metadata_only",
                }),
        );
        self.registered_tool_names.clear();
        self
    }
}

/// Runtime policy for generated MCP edit-session tools.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpFormEditorOptions {
    session_limit: Option<usize>,
    session_idle_timeout: Option<Duration>,
}

impl McpFormEditorOptions {
    /// Create editor options using the default bounded session pool.
    pub const fn new() -> Self {
        Self {
            session_limit: Some(DEFAULT_EDITOR_SESSION_LIMIT),
            session_idle_timeout: Some(DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT),
        }
    }

    /// Limit the number of active edit sessions retained for one form.
    ///
    /// Opening a session beyond this limit evicts the oldest active sessions.
    /// A zero value is clamped to one because opening a session should always
    /// return a readable session.
    pub fn with_session_limit(mut self, session_limit: usize) -> Self {
        self.session_limit = Some(session_limit.max(1));
        self
    }

    /// Keep all sessions until the client closes them.
    pub const fn without_session_limit(mut self) -> Self {
        self.session_limit = None;
        self
    }

    pub const fn session_limit(self) -> Option<usize> {
        self.session_limit
    }

    /// Expire edit sessions that have not been accessed for this duration.
    ///
    /// Expiry is enforced lazily before editor operations. `Duration::ZERO`
    /// expires a session before the next operation after it is opened.
    pub const fn with_session_idle_timeout(mut self, session_idle_timeout: Duration) -> Self {
        self.session_idle_timeout = Some(session_idle_timeout);
        self
    }

    /// Keep idle sessions until the client closes them or the session cap
    /// evicts them.
    pub const fn without_session_idle_timeout(mut self) -> Self {
        self.session_idle_timeout = None;
        self
    }

    pub const fn session_idle_timeout(self) -> Option<Duration> {
        self.session_idle_timeout
    }
}

impl Default for McpFormEditorOptions {
    fn default() -> Self {
        Self::new()
    }
}
