[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg)](https://codecov.io/github/stayhydated/gpui-form)
[![Docs](https://docs.rs/gpui-form/badge.svg)](https://docs.rs/gpui-form/)
[![Crates.io](https://img.shields.io/crates/v/gpui-form.svg)](https://crates.io/crates/gpui-form)

# gpui-form

`gpui-form` is a type-safe form-generation ecosystem for `gpui` and
[`gpui-component`](https://github.com/longbridge/gpui-component), centered on
`#[derive(GpuiForm)]`.

It is designed for three things:

1. Compile-time generation of strongly typed form state and helper types.
1. Concise field annotations on normal application structs.
1. Runtime helpers, metadata, and prototyping support around the derive-based
   workflow.

## Upstream Versions

| `gpui-form` | `gpui-component` | `gpui` |
| :---------- | :--------------- | :----- |
| **git** | | |
| `branch = "master"` | default branch | `rev = "b077f41a9f26ae5ed7fadfea55a501d34afb25de"` |

## Installation

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "b077f41a9f26ae5ed7fadfea55a501d34afb25de" }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }

gpui-form = "*"

# Optional: runtime component helpers, built-in component shapes,
# and the InfiniteSelect derive
gpui-form-component = { version = "*", features = ["component-shape", "derive"] }

# Optional: ready-made component shapes and select derive support
gpui-form-collection = "*"
gpui-form-collection-derive = "*"

# Optional: GPUI component-shape macro/runtime for custom reusable shapes
component-shape-gpui = "*"

# Optional: inventory registration for prototyping/code generation
# gpui-form = { version = "*", features = ["inventory"] }

# Optional: experimental MCP submit integration
# gpui-form = { version = "*", features = ["mcp"] }

# Optional: MCP schema support for chrono and decimal values
# gpui-form = { version = "*", features = ["mcp", "chrono", "rust_decimal"] }

# Optional: headless MCP-only forms without the GPUI runtime re-export
# gpui-form = { version = "*", default-features = false, features = ["derive", "mcp"] }
```

## Quick Start

Collection-backed selects use `SelectItem` from
`gpui-form-collection-derive`.

```rs
use gpui_form::GpuiForm;
use gpui_form_collection_derive::SelectItem;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    France,
    Japan,
}

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub username: Option<String>,

    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub age: Option<u32>,

    #[gpui_form(
        component(gpui_form_collection::select::Select::<_>),
        default = Country::France
    )]
    pub country: Country,

    #[gpui_form(component(gpui_form_collection::checkbox::Checkbox))]
    pub subscribe: bool,
}
```

`#[derive(GpuiForm)]` generates the typed form support around that struct:

- `UserProfileFormFields` for GPUI entity state
- `UserProfileFormComponents` constructors for those fields
- `UserProfileFormValueHolder` for typed editing, defaults, validation, and
  conversion back into the original model

## Component Syntax

`gpui-form` parses contract-backed component expressions. The short form places
the component shape as a direct shape argument:

- `#[gpui_form(component(my::Shape))]`
- `#[gpui_form(component(gpui_form_collection::input::Input::<_>))]`
- `#[gpui_form(component(gpui_form_collection::select::Select::<_>))]`
- `#[gpui_form(component(gpui_form_collection::select::Select::<_>.searchable(true)))]`
- `#[gpui_form(component(gpui_form_collection::select::Select::<_>.from(SelectArgs::builder().searchable(true).build())))]`
- `#[gpui_form(component(gpui_form_collection::combobox::Combobox::<Country>))]`
- `#[gpui_form(component(gpui_form_collection::checkbox::Checkbox))]`
- `#[gpui_form(component(gpui_form_collection::switch::Switch))]`
- `#[gpui_form(component(gpui_form_collection::number_input::NumberInput::<_>))]`
- `#[gpui_form(component(gpui_form_collection::slider::Slider))]`
- `#[gpui_form(component(gpui_form_collection::color_picker::ColorPicker))]`
- `#[gpui_form(component(gpui_form_collection::date_picker::DatePicker))]`
- `#[gpui_form(component(gpui_form_collection::date_picker::DateRangePicker))]`
- `#[gpui_form(component(gpui_form_collection::otp_input::OtpInput::<_>))]`
- `#[gpui_form(component(gpui_form_component::date_picker::DatePicker))]`
- `#[gpui_form(component(gpui_form_component::date_picker::DateRangePicker))]`
- `#[gpui_form(component(gpui_form_component::file_picker::FilePicker))]`
- `#[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>))]`
- `#[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>.searchable(true)))]`

The `gpui_form_component` shape entries above require that crate's
`component-shape` feature. Infinite-select field types also need the
`InfiniteSelect` derive, available through the same crate's `derive` feature or
by depending on `gpui-form-component-derive` directly.

The `component(...)` value starts with a Rust shape path. Plain paths delegate
runtime construction to `GpuiComponentShape::new`; configured expressions such
as `Select::<_>.searchable(true)`,
`Select::<_>.from(SelectArgs::builder().searchable(true).build())`, or
`InfiniteSelect::<_>.searchable(true)` use the expression as a
`GpuiComponentShapeBuilder` for the same base shape. The shape type must be
declared with `component_shape_gpui::component_shape!` or
`#[derive(component_shape_gpui::GpuiComponentShape)]`; hand-written
`GpuiComponentShape` impls are not accepted by `#[derive(GpuiForm)]`.
Shape declarations own render component metadata and prototyping suffix
metadata.

Component shapes own the default value-storage policy for non-optional fields.
Shapes that can safely synthesize a default value, such as the built-in inputs,
selects, toggles, sliders, OTP inputs, file picker, and infinite select, keep
generated value-holder storage as `T`. Shapes that require a present value use
`Option<T>` storage; generated `validate()` reports a missing value and
conversion back to the source model fails when the value is missing. Fields
that use `try_into_source = ...` or `value(koruma_newtype)` can also fail
source reconstruction. Those fallible holder-to-model paths expose
`holder.try_into_original()`; infallible paths expose `holder.into_original()`
and implement `From` when the derive can prove infallibility directly, such as
optional fields, direct fields, or shape-backed fields with a declared field
default. Shape-backed fields without a declared field default keep the checked
`try_into_original()` API even when the reusable shape implements
`GpuiFormComponentShapePolicy` with `DirectValueStorage`.

Common field-level helpers:

- Every field must choose exactly one intent: a component shape,
  `hidden`, or `skip`.
- `#[gpui_form(hidden)]` keeps a field in the generated value holder without
  generating a GPUI component.
- `#[gpui_form(hidden(default = <expr>))]` seeds a hidden generated value holder
  field.
- `#[gpui_form(component(<shape>, default = <expr>))]` seeds a
  component-backed generated value holder field.
- `#[gpui_form(skip)]` excludes a field from generated form widgets and allows
  prefill from the original model. It cannot be combined with
  component or hidden intent on the same field. Holders with skipped source
  fields expose typed `present_fields()` snapshots for debug/preview UI, and
  require `into_original(skipped_value, ...)` for reconstruction.
- `value(type = <form_type>, from_source = <expr>, into_source = <expr>)`
  lets the generated form edit a type that differs from the original field type.
  Use `try_into_source = <expr>` instead of `into_source = <expr>` when the
  form-to-source conversion returns `Result<Source, Error>` with
  `Error: Debug`.
  `type = ...` is the form-side base value type, so write `type = T` rather
  than `type = Option<T>`; source field optionality controls holder storage.
  Put it inside the field intent:
  `#[gpui_form(component(<shape>, value(type = <form_type>, from_source = <expr>, into_source = <expr>)))]`
  or `#[gpui_form(hidden(value(type = <form_type>, from_source = <expr>, into_source = <expr>)))]`.
  Defaults are also intent-scoped, for example
  `#[gpui_form(component(<shape>, value(...), default = <expr>))]`.
- `value(koruma_newtype)` edits the inner value of a struct-level
  `#[koruma(newtype)]` field using `NewtypeValue::into_inner` and reconstructs
  the source model with `NewtypeTryFromInner::try_from_inner`. This works for
  private tuple or named newtype fields and keeps invalid inner values on the
  checked `try_into_original()` / MCP validation path.
- `gpui_form_collection::input::Input::<_>` parses non-`String` form-side
  value types with `FromStr` in prototyping output, so value objects can use
  `value(type = ..., from_source = ..., into_source = ...)` while the source
  model keeps its storage type.
- `gpui_form_collection::input::ParsedInput::<_, Config>` keeps the same input
  widget but lets applications provide a typed parser, formatter, placeholder,
  empty-as-clear policy, and optional widget-level validation when `FromStr`
  and `ToString` are not the right UI contract.
- `gpui_form_collection::number_input::NumberInput::<_>` uses `FromStr` for
  non-`String` form-side values when parsing edits.
- `gpui_form_collection::otp_input::OtpInput::<_>` also uses `FromStr` for
  non-`String` form-side values.
- Generic component expressions use Rust expression turbofish syntax, such as
  `Input::<_>`, `Select::<_>.searchable(true)`,
  or `Select::<_>.from(SelectArgs::builder().searchable(true).build())`. The
  derive normalizes the base shape path and resolves `_` to the field's
  form-side type.
- generated `FormFields` members and `FormComponents` constructors use the
  source field identifier, such as `birth_date`. Derive-generated inventory
  records shape-level `field_suffix = "..."` metadata when available, or
  a resolved shape-name fallback, for prototyping-specific DOM IDs, event
  handlers, and helper names such as `on_birth_date_date_picker_event`.
  Shape-level suffixes must be non-empty ASCII identifier suffixes; direct
  `component_shape::ComponentPrototyping::field_suffix(...)` calls validate the
  same contract in const evaluation.
- Field-level `#[koruma(...)]` attributes are accepted by `GpuiForm` and copied
  onto the generated value holder, including fields that use intent-scoped
  `value(type = ..., from_source = ..., into_source = ...)` or
  `try_into_source = ...` to validate a form-side type.

`gpui_form_component::infinite_select::InfiniteSelect::<_>` expects the
field type to derive `gpui_form_component::InfiniteSelect`, which implements
the runtime `gpui_form_component::infinite_select::InfiniteSelectValue` trait
for the enum tree.
The enum tree must also implement `PartialEq` because the backing
`gpui-component` select compares selected values.
Use `gpui-form-component` with its `derive` feature or import
`gpui-form-component-derive` explicitly.
Enable `gpui-form-component`'s `component-shape` feature when using
`InfiniteSelect::<_>` directly as a form shape.
Use `InfiniteSelect::<_>.searchable(true)` when the cascading selects should
enable search; use plain `InfiniteSelect::<_>` for default options.

Common struct-level helpers:

- `#[gpui_form(empty)]` marks an intentional empty form.
- `#[gpui_form(koruma)]` enables Koruma-backed validation wiring.
- `#[gpui_form(koruma(fluent))]` enables Koruma validation plus fluent error
  rendering.

## Experimental MCP Submit

Enable the facade `mcp` feature to generate typed MCP submit metadata for forms
that opt in with `#[gpui_form(mcp)]` (or `#[gpui_form(mcp(...))]`). Those
forms get `gpui_form::mcp::McpForm` and `McpEditableForm` implementations
beside the normal GPUI form types.
The feature implies inventory registration and routes through the
`gpui-form-mcp` integration crate. `#[gpui_form(mcp)]` requires the `mcp`
feature and cannot be combined with `#[gpui_form(no_inventory)]` or generic
form types, because generated MCP tools are discovered through inventory.

```toml
[dependencies]
gpui-form = { version = "*", default-features = false, features = ["derive", "mcp"] }
serde = { version = "1", features = ["derive"] }
```

Enable `chrono` or `rust_decimal` too when MCP-visible form fields or handler
response types use `chrono` dates/times or `rust_decimal::Decimal`; those
features publish the matching `gpui_form::mcp::McpJsonSchema` impls.

Keep the default features, or enable `runtime`, when the same crate also
renders component-backed GPUI forms or references `gpui_form::runtime`.

```rs
use gpui_form::GpuiForm;
use serde::{Deserialize, Serialize};

/// Submit a contact request from structured MCP arguments.
#[derive(Clone, Debug, Deserialize, GpuiForm, Serialize)]
#[gpui_form(mcp(
    name = "submit_contact",
    title = "Submit contact request"
))]
pub struct ContactRequest {
    #[gpui_form(hidden)]
    pub email: String,
}

#[derive(Debug, gpui_form::mcp::McpJsonSchema, Serialize)]
pub struct ContactSubmitResponse {
    pub email: String,
}

#[gpui_form::mcp_submit]
async fn submit_contact(request: ContactRequest) -> Result<ContactSubmitResponse, String> {
    Ok(ContactSubmitResponse {
        email: request.email,
    })
}

fn main() -> gpui_form::mcp::ServeStdioResult {
    gpui_form::mcp::serve_stdio_blocking()
}
```

MCP tool calls populate the generated `ContactRequestFormValueHolder`, run
generated holder validation, convert to `ContactRequest` when the holder can
reconstruct the model, and then call the application-owned handler. Forms with
`#[gpui_form(skip)]` fields intentionally do not get model-handler conversion;
give those handlers the generated `FormValueHolder` as their first parameter.
`#[gpui_form::mcp_submit]` infers model submission when the handler takes the
source model. For forms with `#[gpui_form(skip)]` context or other
application-owned context, give the handler the generated `*FormValueHolder`
parameter and the macro registers holder submission instead. This inference is
type-directed through generated `McpSubmitArgument` implementations, not a
name suffix convention.
Submit handlers can be synchronous or async. Return `Result<T, E>` for handler
errors and normal responses, where `T: serde::Serialize + gpui_form::mcp::McpJsonSchema`
and `E: fmt::Display`. Prefer a typed response struct or newtype that derives
`gpui_form::mcp::McpJsonSchema` for precise output schemas, or return
`gpui_form::mcp::McpObject` for dynamic object-shaped JSON.
For submit flows with shared runtime state, declare the context on the form
itself with `#[gpui_form(mcp(context(MyContext), ...))]`, optionally add
`response(MyResponse)` for a precise output schema, and either implement
`gpui_form::mcp::McpContextSubmit<MyContext>` for the source model or add
`submit(path::to::async_fn)` so the derive emits that impl. Generated submit
impls can also use `map_response(path::to::fn)` to convert a raw handler
response into the published response type, and `error(MyError)` to override
the default `String` error type. Use `submitter(path::to::Trait)` instead of
explicit `context(...)` and `submit(...)` options when a trait supplies
associated `Context`, `Response`, and `Error` types plus
`submit_with_context(self, context)`. Pair submitters with `map_response(...)`
when the submitter returns a raw application response that should be converted
into the published MCP response; the trait must be visible enough for the
form's generated impl.
Then call
`gpui_form::mcp::register_context_submitters(&mut server, context)?` or
`register_context_submitters_with_editor_options(...)`. The derive inventories
those forms, so one registration call wires every matching context-backed
submit tool plus `*_edit_submit`; the context type must be
`Clone + Send + Sync + 'static`. If `response(...)` is omitted, the generated
registration uses `McpObject`. Use
`register_context_submitters_strict(...)` or
`register_context_submitters_strict_with_editor_options(...)` when zero
matching context registrations should fail setup instead of succeeding as a
no-op; the strict variants return a registration report with the matched
context registration count.
Use struct-level
`#[gpui_form(mcp(name = "...", title = "...", description = "..."))]` to
override the generated MCP tool name, title, or description. When
`description` is omitted, the derive uses the form type's Rust doc comment.
The same list accepts optional MCP tool annotation hints with
`read_only = ...`, `destructive = ...`, `idempotent = ...`, and
`open_world = ...`, so generated submit tools can advertise whether they
inspect local/server state, mutate it, or call into external systems.
Direct submit tools default to `read_only = false`, `destructive = true`,
`idempotent = false`, and `open_world = true` unless overridden. Add repeated
`icon(src = "...", mime_type = "...", size = "...", theme = "light" | "dark")`
entries to publish MCP tool icons, and `task_support = "forbidden" |
"optional" | "required"` to publish MCP tool execution task support.
`read_only = true` and `destructive = true` cannot be combined.
Use field-level `#[gpui_form(label = "...")]`,
`#[gpui_form(description = "...")]`, and `#[gpui_form(example = "...")]` to
publish form-author intent in MCP schemas and editor snapshots. Field
descriptions infer from `///` rustdoc when no explicit description is present,
labels default from the field name, and examples may be repeated.
The lower-level `McpServer` API remains
available when an application wants to compose form tools with other
`component-shape-mcp` integrations. Use `gpui_form::mcp::server_named(name,
version)?` when generated form handlers and editor tools should advertise
application-owned metadata, then call `.serve_stdio().await` or
`.serve_stdio_blocking()`. Use `gpui_form::mcp::builder()` or
`builder_named(name, version)` when deferred setup is useful. Use
`gpui_form::mcp::serve_stdio_blocking()` for the default stdio server.
Generated servers register `#[gpui_form::mcp_submit]` submit handlers plus
editable holder-session tools for every `#[gpui_form(mcp)]` form. Forms that
also have a submit handler get `*_edit_submit`, so agents can submit a valid
edit session by `session_id`.
Use `gpui_form::mcp::register_with_options(&mut server, McpFormRegistrationOptions::submit_only())`
to publish only generated submit handlers,
`McpFormRegistrationOptions::editor_only()` to publish only editable holder
sessions, or `McpFormRegistrationOptions::metadata_only()` to inspect the
selected tool definitions and skipped-tool report without mutating the server.
`tool_definitions_with_options(...)` returns the profile-selected definitions
directly. `register()` remains equivalent to `McpFormRegistrationOptions::all()`.

For a composed server, such as a binary that also depends on `gpui-table`,
use the shared builder:

```rs
let server = gpui_form::mcp::McpServer::builder("my-app", env!("CARGO_PKG_VERSION"))
    .register(gpui_form::mcp::register)
    .register(gpui_form::mcp::register_prompt_templates)
    // If your binary also enables gpui-table MCP integration:
    .register(gpui_table::mcp::register)
    .build()?;
```

Manual form tools can still be registered directly:
`gpui_form::mcp::form::<ContactRequest>(&mut server).model(handler)?` or
`.holder(handler)?` for `Result<T, E>` handlers and `handler` must return
`Result`.
Use `.model_with_editor(handler)?`, `.model_async_with_editor(handler)?`,
`.holder_with_editor(handler)?`, or `.holder_async_with_editor(handler)?` when
the same form should publish submit, editable holder-session tools, and
`*_edit_submit` for submitting the current edit session through the same
handler.
Use the matching `*_with_editor_options` helpers with
`McpFormEditorOptions` when a server needs a custom session cap.
Registration reports setup errors such as duplicate tool names or generated
resource URIs.

MCP forms can also expose editable holder sessions directly:

```rs
let mut server = gpui_form::mcp::McpServer::new("my-app", env!("CARGO_PKG_VERSION"));
gpui_form::mcp::form::<ContactRequest>(&mut server).editor()?;
```

This registers `*_edit_open`, `*_edit_list`, `*_edit_read`, `*_edit_patch`,
`*_edit_validate`, `*_edit_close`, and `*_edit_close_all` tools derived from
the form's MCP tool name. `edit_open` accepts an optional `values` object,
`edit_patch` accepts a `session_id`, optional `values` object, optional
`clear` field-name array, optional `replace = true` to replace the whole
pending value set instead of merging, and optional `expected_revision` to reject
stale client updates. Tool results include the session `revision`,
agent-supplied `values`, missing required fields, validation `errors`, `valid`,
`fields` metadata with each field's label, description, examples, schema,
current `has_value`/`value` state, and field-level `errors`, and
`submit_arguments` that can be sent to the normal submit tool. The published
snapshot schema keeps `fields[]` typed with a
per-field `oneOf`. Field patches and clears are decoded by rebuilding a cloned
holder before they are committed, so a failed bulk patch does not partially
mutate the session.
`edit_list` returns every active session for the form using the same snapshot
shape plus `session_limit` and `session_idle_timeout_ms`, and
`edit_close` accepts an optional `expected_revision` before closing one
session. `edit_close_all` closes every active session for that form and returns
`closed_count` plus the closed `session_ids`.
Editors retain at most `DEFAULT_EDITOR_SESSION_LIMIT` active sessions per form
by default and expire idle sessions after
`DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT`; opening another session evicts the oldest
session when the cap is exceeded. Use
`.editor_with_options(McpFormEditorOptions::default().with_session_limit(n).with_session_idle_timeout(duration))`,
the matching `*_with_editor_options` helpers, or
`McpFormEditorOptions::default().without_session_limit().without_session_idle_timeout()`
to tune that policy.
`edit_open` and `edit_list` report a `cleanup` object with expired and evicted
session IDs so clients can drop local handles that the server removed while
servicing the request.
When the form is registered with one of the `*_with_editor` submit helpers, or
through a generated `#[gpui_form::mcp_submit]` handler, `*_edit_submit` accepts
a `session_id`, optional `expected_revision`, and optional
`close_on_success = false`, validates the current session, and calls the same
submit handler without copying `submit_arguments` into a second tool call.
Successful edit submits close the session by default only when the session has
not advanced while the handler was running; validation, stale revision, handler
failure, or concurrent patch cases leave it open for repair or retry.
Generated editor tool definitions publish MCP annotations: `edit_list`,
`edit_read`, and `edit_validate` are read-only and idempotent, `edit_open` is a
non-destructive session mutation, `edit_patch`, `edit_close`, and
`edit_close_all` are destructive session mutations, and `edit_submit` is a
destructive open-world submit call.
Patches use the same generated decoding path as submit calls, including
component-shape `MCP_INPUT` schemas and value-storage policies. These tools
edit headless generated value holders; live GPUI widgets still need an
application-owned runtime bridge from the holder/session state into GPUI
entities.

MCP failures keep their text content and also set `structured_content.error`
with a stable `kind` plus relevant fields such as `field`, `value`, `name`, or
`detail`, so agents can inspect decode, validation, session, unknown-tool, and
handler failures without parsing prose. Generated form validation failures also
include a `details` array of structured issue objects with `scope`, `message`,
and, when available, `field`, `validator`, `path`, `target`, `element_index`,
and validator `params`. Editor session `errors` and field-level `errors` use
the same issue object shape.
Generated tool definitions also publish output schemas: submit tools advertise
the handler response type's `McpJsonSchema` or `McpObject` for dynamic JSON,
and editor tools publish strict schemas for session snapshots and close
results. Output schemas must declare root `type: "object"` to match MCP
structured content.
Generated MCP servers also advertise `resources` and publish JSON resources for
every registered form:
`gpui-form://forms/{tool_name}/descriptor`,
`gpui-form://forms/{tool_name}/schema`, and
`gpui-form://forms/{tool_name}/examples`. The descriptor resource contains the
form name, source module path, tool metadata, resource links, field labels,
descriptions, examples, required/default/storage metadata, component MCP input
metadata, validation rules, and per-field schemas. The schema resource contains
the submit input JSON Schema, and the examples resource groups field examples
for clients that want lightweight prompt or repair context. Direct per-form
registration helpers and inventory registration publish the same resources;
`McpFormRegistrationOptions::metadata_only()` remains non-mutating.
Prompt templates are opt-in. Call
`gpui_form::mcp::register_prompt_templates(&mut server)?` after registering
generated form tools, `register_context_submitter_prompt_templates::<Context>(...)`
for context-backed submitters, or
`register_form_prompt_templates::<Form>(&mut server)?` for one form. The
generated prompt names are `fill_{tool_name}_form`,
`repair_{tool_name}_form`, and `submit_{tool_name}_form`; they point clients
at the descriptor/schema/examples resources and the matching edit tools when
iterative repair is useful. Prompt registration publishes the same form
resources first if they are not already registered.

Generated MCP input schemas use the field type's `McpToolValue` schema, so the
published schema and generated decoder stay paired. Component-backed fields
also attach value-specific shape metadata from
`<Shape as ComponentShapeFor<Field>>::MCP_INPUT` when it is available.
When Koruma validation is enabled, generated field schemas attach
`x-gpuiFormValidation` rule metadata. Literal `LenValidation`,
`RangeValidation`, and `NonEmptyValidation` arguments are also reflected as
JSON Schema hints such as `minLength`, `maxLength`, `minimum`,
`maximum`, `exclusiveMinimum`, `exclusiveMaximum`, `minItems`, and `maxItems`
when the field schema type is unambiguous.
Generated MCP validation errors use gpui-form's field rule metadata first and
fall back to Koruma's generic `ValidationIssues` output when no field-specific
rule extractor is available.
`component_shape_gpui` infers common value-specific MCP input from declared
value metadata; custom or ambiguous wire schemas should use a field type that
implements `gpui_form::mcp::McpToolValue`. The blanket implementation covers
`Deserialize` types that implement or derive `McpJsonSchema`; use
`gpui_form::mcp::McpAny` when a typed field intentionally accepts
unconstrained JSON. Aliases inherit the schema of their target type, fixed
tuples with 1 to 4 elements publish exact array schemas, and tuple or named
transparent newtypes, named structs, or fieldless enums can derive the schema
trait directly through `gpui_form::mcp`.
In crates that also depend on another MCP facade such as `gpui-table`, add
`#[mcp(crate = gpui_form::mcp)]` to custom `McpJsonSchema` or `McpToolInput`
derives so generated schema impls target the gpui-form facade.
The derive follows serde deserialize names, records field aliases in
`x-mcpAliases`, includes enum aliases, skips
deserialization-skipped fields, rejects flattened fields, and treats
serde-defaulted fields as not required. Use `gpui_form::mcp::McpRange<T>` for
typed `{ "min": ..., "max": ... }` range arguments. Custom top-level MCP tool
argument structs can also derive `gpui_form::mcp::McpToolInput` through the
facade when composing manual typed tools; that derive also implements
`McpJsonSchema`, so object inputs can be reused as field values.

This integration is MCP-only. It does not add `gpui-form-cli`, `clap`, shell
completion, shell argument parsing, or exit-code behavior. The crate includes a
stdio MCP server backed by the official `rmcp` SDK for local MCP clients, but
it does not start with GPUI app startup or instantiate GPUI widgets to execute
submissions.

See `examples/mcp-submit` for a runnable stdio MCP server:

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"manual","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}' \
  '{"jsonrpc":"2.0","id":3,"method":"prompts/list","params":{}}' \
  | cargo run -p mcp-submit
```

See `examples/mcp-form-table` for a headless composed server that declares a
custom component shape, exposes a form submit tool, and composes it with a
`gpui-table` MCP query tool:

```sh
cargo run -p mcp-form-table
```

## Infinite Select Runtime

Infinite-select shape-backed fields are backed by
`gpui_form_component::infinite_select::InfiniteSelectState`, which owns the
root and child `SelectState`s, exposes render-ready level snapshots, and emits
a single typed change event with the rebuilt nested value, both path forms, the
previous paths, and the changed depth.

```rs
use gpui_form_component::infinite_select::{InfiniteSelectEvent, InfiniteSelectState};

let location = cx.new(|cx| {
    InfiniteSelectState::new(Country::default(), window, cx)
});

cx.subscribe_in(
    &location,
    window,
    |_, _, event: &InfiniteSelectEvent<Country>, _, _| {
        let _value = event.value();
        let _path = event.path();
        let _key_path = event.key_path();
        let _previous_key_path = event.previous_key_path();
        let _changed_depth = event.changed_depth();
    },
);
```

Rendering code can stay on the runtime helper instead of combining select
handles with separate label lookups:

```rs
for field in location.read(cx).form_fields() {
    let _ = field;
}
```

The derive/runtime pair also exposes typed option labels, stable key paths, and
typed path errors:

- root option titles come from `variant_label()` instead of raw `variant_name()`
- `#[fluent_kv(keys = ["label", "description"], keys_this)]` emits
  `es-fluent` variant and type metadata for application-owned localizers;
  generated runtime labels use plain fallback names because the runtime trait
  contract is localizer-free
- `#[tuple_enum(key = "...")]` overrides persisted keys when enum names should
  stay decoupled from storage
- `selection_key_path()` / `build_from_key_path(...)` round-trip nested values
  without depending on enum ordering
- `InfiniteSelectKeyPath` supports `Display`, `FromStr`, and serde string
  round-trips for URLs and persisted config
- `build_from_path(...)`, `build_from_key_path(...)`, `set_path(...)`, and
  `set_key_path(...)` return `InfiniteSelectPathError` with the failing depth
  plus the invalid key/index segment
- `set_selected_index_at_depth(...)` / `set_selected_key_at_depth(...)` support
  incremental programmatic updates

## Validation With Koruma

`gpui-form` can mirror Koruma validation metadata into the generated value
holder so your form state and your domain model stay aligned.

```rs
use gpui_form::GpuiForm;
use koruma::{Koruma, KorumaAllFluent};
use koruma_collection::{
    collection::NonEmptyValidation,
    numeric::RangeValidation,
};

#[derive(Clone, Debug, GpuiForm, Koruma, KorumaAllFluent)]
#[gpui_form(koruma(fluent))]
pub struct Signup {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    #[koruma(NonEmptyValidation::<_>)]
    pub username: String,

    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    #[koruma(RangeValidation::<_>.min(18).max(120))]
    pub age: Option<u32>,
}
```

When validation is enabled:

- required-value semantics are preserved when the generated holder represents
  required fields as `Option<T>`
- direct Koruma validator chains are mirrored
- generated value-holder validation uses the same validator set as the source
  struct

## Component Shapes

There are three supported component-shape workflows.

### 1. Use a collection component

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct Account {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub code: AccountCode,
}
```

The `_` generic is resolved to the field's form-side type, including any
intent-scoped `value(type = ...)` override. `Combobox<T>` is the exception: its
generic is the selected item type, so a `Vec<Country>` field uses
`Combobox::<Country>`. The collection combobox publishes
`GpuiFormComponentShapePolicy::ValueStoragePolicy = DirectValueStorage`: an empty selection is emitted as
`ValueChange::Clear`, so optional fields clear to `None` and
non-optional `Vec<T>` fields reset to their declared
intent-scoped `default = ...` when present, otherwise `Vec::default()`.

### 2. Derive on an owned component

```rs
use gpui_form::GpuiForm;
use component_shape_gpui::GpuiComponentShape;

#[derive(GpuiComponentShape)]
#[gpui_component_shape(value = Vec<String>, field_suffix = "input")]
pub struct TagsInput {
    state: gpui::Entity<TagsInputState>,
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for TagsInput {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

pub struct TagsInputState;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct PostEditor {
    #[gpui_form(component(TagsInput))]
    pub tags: Option<Vec<String>>,
}
```

`new` accepts a constructor expression. Use a function path or closure when the
macro should pass `(window, cx)` for you, or use a full constructor expression
such as `TagsInputState::with_label(window, cx, "tags")` when the expression
should be emitted as written. If `new` is omitted, the derive calls
`<State>::new(window, cx)`. When `state = ...` is omitted, the derive infers a
same-module state named after the component, such as `TagsInputState`; use
explicit `state = ...` when the backing state does not follow that convention.

### 3. Declare a reusable external shape

```rs
component_shape_gpui::component_shape! {
    pub struct EmailInputShape {
        state = gpui_component::input::InputState;
        component = gpui_component::input::Input;
        value = String;
        field_suffix = "input";
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for EmailInputShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct ContactForm {
    #[gpui_form(component(EmailInputShape))]
    pub email: String,
}
```

`component_shape!` creates a local zero-sized shape type, so downstream crates
can attach the `gpui-form` contract to external component state without running
into Rust's orphan rules.
It uses `state = ...` for the wrapped external state type, plus `new`,
`component`, `value = ...`, `values(...)`, `value_binding`, `mcp_input = ...`,
and `field_suffix` metadata. If `new` is omitted, the macro calls
`<State>::new(window, cx)`.
For MCP submit schemas, common shape metadata is inferred from unambiguous
declared values such as `String`, booleans, numbers, dates, `Vec<T>`,
set-like primitive collections, fixed arrays, `gpui_form::mcp::McpRange<T>`,
`gpui_form::mcp::McpAny`, or `(Option<T>, Option<T>)` ranges. `Vec<T>` and
fixed arrays publish list metadata; set-like collections publish set metadata.
Inferred metadata is
attached to the generated `ComponentShapeFor<Value>` impl, so multi-value
shapes can expose different MCP input metadata for different supported value
types. Use `mcp_input = string`, `mcp_input = object`, or another
`McpInput` expression when a generic or custom shape knows its model-facing MCP
input better than the value type can be inferred. MCP descriptors store a typed
schema function for every visible form field using `gpui_form::mcp::McpToolValue`,
which gives `Deserialize` field types that implement or derive `McpJsonSchema`
strict decode plus aliases, fixed tuples with 1 to 4 elements, `McpAny`,
derived tuple or named transparent newtypes, named structs, and fieldless enums
precise schemas.
Unsupported MCP-visible field types fail at compile time.
`component = ...` must be a path-like type, `mcp_input = ...` accepts common
constructor shorthands or a `McpInput` expression, and `field_suffix = "..."`
must be a non-empty ASCII identifier suffix.
Separate metadata entries with semicolons.
The block may also contain `impl` items. `value = ...` and `values(...)` emit
`GpuiComponentShapeFor<T>` impls for simple supported form-side value types.
Manual `GpuiComponentShapeFor<T>` impls are rejected inside `component_shape!`.
A nested `GpuiComponentValueBinding<T>` impl is emitted with the shape. When no
explicit `value = ...` or `values(...)` metadata is present, that nested impl
also publishes `T` plus shape-level value-binding metadata for prototyping
subscriptions.

Generated `GpuiForm` component fields compile against
`gpui_form::runtime::shape`, which is re-exported by the facade crate. Normal
application crates do not add `gpui-form-runtime` just because a field uses a
component shape. Add `gpui-form-runtime` directly only for lower-level shape
crates that use `#[derive(GpuiComponentShape)]`, `component_shape!`, or direct
runtime shape contracts; those proc macros emit direct runtime paths and resolve
renamed runtime dependencies.
If a derived shape uses `value_binding` and omits `value = ...`/`values(...)`,
value compatibility is inferred from matching
`GpuiComponentStateValueBinding<Value>` impls on the backing state. Without
`value_binding`, provide `value = ...`/`values(...)` metadata or manual
`GpuiComponentShapeFor<Value>` impls outside the derive. The GPUI shape
declaration owns state, render, value-type, and value-binding metadata. The
companion `GpuiFormComponentShapePolicy` impl owns form value-holder storage.

Component-derived shapes can opt into generated value synchronization by adding
`value_binding` to `#[gpui_component_shape(...)]` and implementing
`GpuiComponentStateValueBinding<T>` on the backing state:

```rs
pub struct TagsInputState;

impl gpui_form_runtime::shape::GpuiComponentStateValueBinding<Vec<String>> for TagsInputState {
    type Event = TagsInputEvent;
    /* seed_value_binding_state and value_change */
}
```

Wrapper shapes can put the binding impl directly inside `component_shape!`:

```rs
component_shape_gpui::component_shape! {
    pub struct EmailInputShape {
        state = gpui_component::input::InputState;
        component = gpui_component::input::Input;

        impl gpui_form_runtime::shape::GpuiComponentValueBinding<String> for EmailInputShape {
            type Event = gpui_component::input::InputEvent;
            /* seed_value_binding_state and value_change */
        }
    }
}
```

Generated form code reaches shape contracts through
`gpui_form::runtime::shape`. Lower-level shape crates may also depend on
`gpui-form-runtime` and use `gpui_form_runtime::shape` directly. Concrete
runtime widgets are available from `gpui_form_component`; `gpui-form` does not
re-export component modules or their derives.

## File Picker Runtime

For manual native path selection, use `gpui_form_component::file_picker`. The
runtime uses GPUI's
`PathPromptOptions` from the pinned Zed git dependency and renders the control
with `gpui-component` buttons, icons, sizing, and theme tokens.
Built-in defaults are plain English fallback copy. When a form needs localized
placeholder, prompt, or button text, render those messages through an
application-owned `es-fluent` localizer and pass the resulting strings through
`placeholder(...)`, `prompt(...)`, and `browse_label(...)`. The runtime ships
Fluent resources for callers that localize those messages explicitly.

```rs
use gpui_form_component::file_picker::{FilePicker, FilePickerEvent, FilePickerState};

let picker = cx.new(|cx| FilePickerState::new(window, cx));

cx.subscribe_in(&picker, window, |_, _, event: &FilePickerEvent, _, _| {
    if let FilePickerEvent::Change(paths) = event {
        let _paths = paths;
    }
});

FilePicker::new(&picker)
    .placeholder("Choose a file")
    .prompt("Choose a file")
    .cleanable(true);
```

Enable `gpui-form-component`'s `component-shape` feature when using
`FilePicker` directly in
`#[gpui_form(component(gpui_form_component::file_picker::FilePicker))]`;
the shape stores `FilePickerState` as its backing entity state.

## Date Conversion

`date_picker` fields can edit a different form-side type than the original model
field with an intent-scoped `value(...)` conversion:

```rs
#[derive(Clone, Debug, gpui_form::GpuiForm)]
pub struct User {
    #[gpui_form(
        component(
            gpui_form_collection::date_picker::DatePicker,
            value(
                type = chrono::NaiveDate,
                from_source = to_form_date,
                into_source = to_model_timestamp,
            )
        )
    )]
    pub birth_date: Timestamp,
}
```

This pattern is useful when the model stores a domain-specific timestamp type
but the UI should edit a calendar date. With no `default = ...`, non-optional
date fields start empty in the generated holder and are still required when
calling `try_into_original()` or generated `validate()`.
The collection `DatePicker` and `DateRangePicker` shapes use the upstream
`gpui_component::date_picker::DatePicker` runtime. Range fields use
`gpui_component::date_picker::DatePickerState::range` under the hood.

## Prototyping

Enable the `inventory` feature when you want to generate scaffolding from
registered `GpuiFormShape` metadata:

```rs
use gpui_form::schema::registry::{GpuiFormShape, inventory};
use gpui_form_prototyping_core::FormShapeAdapter;

for shape in inventory::iter::<GpuiFormShape>() {
    let parts = FormShapeAdapter::new(shape)
        .parts()
        .expect("shape metadata should be valid");

    let _imports = parts.imports;
}
```

See [`examples/prototyping`](examples/prototyping) for a complete generator
that reads shape inventory, clears stale generated form modules, writes
scaffolded GPUI form files into `examples/some-lib-forms/src/forms`, mirrors
them under `examples/prototyping/output`, and formats both with `rustfmt`. The
Storybook form titles generated by that example use the GPUI app context so
they follow the active Storybook locale through `gpui-es-fluent`. Value-bound
shape-backed fields use helper functions and runtime aliases from
`gpui_form::runtime::shape` for state and event projections. Component entity
fields use source field names such as `email`, while helper signatures keep
suffix metadata in names such as `on_email_input_event`. Shape-backed
prototyping fails with a field-specific
`PrototypingError` when required render, value-binding, shape path, or default
storage metadata is missing instead of generating placeholder UI. The adapter
uses the derive-emitted holder conversion shape metadata so generated debug
output matches the holder's `into_original`, `try_into_original`, or
skipped-field reconstruction API.
Generic forms are not registered in inventory because inventory metadata must
name one concrete source type and module path. When the `inventory` feature is
enabled, add `#[gpui_form(no_inventory)]` to generic forms that derive
holders/components without registering prototyping metadata. MCP submit forms
cannot use `no_inventory` or generic parameters.

## Examples

[`examples/README.md`](examples/README.md) is the canonical index for runnable
workspace examples.

- `cargo run -p some-lib-forms`: browse generated forms in a storybook-style UI
- `cargo run -p gpui-form-component-story`: browse runtime component demos
- `cargo run -p prototyping`: regenerate the `some-lib-forms` scaffolded GPUI
  form files and the checked-in `examples/prototyping/output` mirror from shape
  inventory
- `cargo run -p mcp-submit`: serve generated form submit/edit tools,
  resources, and prompt templates over stdio MCP
- `cargo run -p mcp-form-table`: run the composed form submit plus table-filter
  MCP server
