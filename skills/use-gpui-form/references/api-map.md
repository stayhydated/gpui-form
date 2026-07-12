# gpui-form User API Map

Use this reference for application code that consumes `gpui-form`.

## Install Shape

Use `gpui-form` as the public entry point. Match `gpui` and `gpui-component`
versions to the version guidance for the `gpui-form` version in use.

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "b077f41a9f26ae5ed7fadfea55a501d34afb25de" }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
gpui-form = "*"
gpui-form-component = { version = "*", features = ["component-shape", "derive"] }
gpui-form-collection = "*"
gpui-form-collection-derive = "*"
component-shape-gpui = "*"
gpui-form-runtime = "*"

# Optional experimental MCP submit integration:
# gpui-form = { version = "*", features = ["mcp"] }

# Headless MCP-only forms can skip the GPUI runtime re-export:
# gpui-form = { version = "*", default-features = false, features = ["derive", "mcp"] }
```

## Imports

Use the facade for `GpuiForm` and import component derives explicitly:

```rust
use gpui_form::GpuiForm;
use gpui_form_collection_derive::SelectItem;
use gpui_form_component::InfiniteSelect;
use component_shape_gpui::GpuiComponentShape;
```

Useful runtime/helper paths:

- `gpui_form::runtime::shape`
- `gpui_form::mcp` when the `gpui-form` `mcp` feature is enabled
- `gpui_form_component::date_picker`
- `gpui_form_component::file_picker`
- `gpui_form_component::infinite_select`
- `gpui_form::core::numeric`
- `component_shape_gpui::component_shape!`

Generated `GpuiForm` component fields use `gpui_form::runtime::shape` through
the facade when the `runtime` feature is enabled. Add `gpui-form-runtime`
directly only when defining lower-level shapes with `component_shape_gpui`
shape macros or direct runtime value-binding impls.

## Supported Component Syntax

```rust
#[gpui_form(component(gpui_form_collection::input::Input::<_>))]
#[gpui_form(component(gpui_form_collection::select::Select::<_>))]
#[gpui_form(component(gpui_form_collection::select::Select::<_>.searchable(true)))]
#[gpui_form(component(gpui_form_collection::select::Select::<_>.from(
    SelectArgs::builder().searchable(true).build()
)))]
#[gpui_form(component(gpui_form_collection::combobox::Combobox::<Item>))]
#[gpui_form(component(gpui_form_collection::number_input::NumberInput::<_>))]
#[gpui_form(component(gpui_form_collection::slider::Slider))]
#[gpui_form(component(gpui_form_collection::color_picker::ColorPicker))]
#[gpui_form(component(gpui_form_collection::date_picker::DatePicker))]
#[gpui_form(component(gpui_form_collection::date_picker::DateRangePicker))]
#[gpui_form(component(gpui_form_collection::otp_input::OtpInput::<_>))]
#[gpui_form(component(gpui_form_component::date_picker::DatePicker))]
#[gpui_form(component(gpui_form_component::date_picker::DateRangePicker))]
#[gpui_form(component(gpui_form_component::file_picker::FilePicker))]
#[gpui_form(component(gpui_form_collection::checkbox::Checkbox))]
#[gpui_form(component(gpui_form_collection::switch::Switch))]
#[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>))]
#[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>.searchable(true)))]
#[gpui_form(component(my::Shape))]
```

Custom `my::Shape` types must be declared with
`component_shape_gpui::component_shape!` or
`#[derive(component_shape_gpui::GpuiComponentShape)]`. Put render component
metadata and `field_suffix = "..."` metadata on the shape declaration, and implement
`gpui_form_runtime::shape::GpuiFormComponentShapePolicy` for form holder
storage.

Common field attributes:

```rust
#[gpui_form(component(<shape>))]
#[gpui_form(hidden)]
#[gpui_form(component(<shape>, default = <expr>))]
#[gpui_form(hidden(default = <expr>))]
#[gpui_form(component(<shape>, value(type = <form_type>, from_source = <expr>, into_source = <expr>)))]
#[gpui_form(component(<shape>, value(type = <form_type>, from_source = <expr>, try_into_source = <expr>)))]
#[gpui_form(hidden(value(type = <form_type>, from_source = <expr>, into_source = <expr>)))]
#[gpui_form(hidden(value(koruma_newtype)))]
#[gpui_form(skip)]
```

Configured shape expressions, such as `Select::<_>.searchable(true)` or
`Select::<_>.from(SelectArgs::builder().searchable(true).build())`, may be
used anywhere `<shape>` appears when the expression returns a
`GpuiComponentShapeBuilder` for the same base shape.

Every field must choose exactly one intent: component, `hidden`, or `skip`.
Use `hidden` for value-holder-only fields. `skip` cannot be combined with
component or hidden intent on the same field. `value(type = ...)` expects the
form-side base value type, so use `type = T`, not `type = Option<T>`, even when
the source field is optional. Use `try_into_source = ...` when the reverse
conversion returns `Result<Source, Error>` with `Error: Debug`, and use
`value(koruma_newtype)` for
struct-level `#[koruma(newtype)]` fields that should edit and validate their
inner value.

Common struct attributes:

```rust
#[gpui_form(empty)]
#[gpui_form(koruma)]
#[gpui_form(koruma(fluent))]
```

## Component Selection

- Use `gpui_form_collection::input::Input::<_>` for text-like and
  `FromStr`-parsable values.
- Use `gpui_form_collection::input::ParsedInput::<_, Config>` when a text-like
  value needs an application-owned parser, formatter, placeholder,
  empty-as-clear policy, or widget-level validation.
- Use `gpui_form_collection::checkbox::Checkbox` or
  `gpui_form_collection::switch::Switch` for `bool` fields.
- Use `gpui_form_collection::select::Select::<_>` for a single enum-like
  choice; derive `SelectItem`. Use a configured shape expression such as
  `Select::<_>.searchable(true)` when the field needs custom construction.
- Use `gpui_form_collection::combobox::Combobox::<Item>` for multi-value enum-like
  choices from `gpui_component::combobox::Combobox`. Empty selection is
  `FormValueChange::Clear`; optional fields clear to `None`, and non-optional
  `Vec<Item>` fields reset to their intent-scoped `default = ...`
  when present, otherwise `Vec::default()`.
- Use `gpui_form_collection::number_input::NumberInput::<_>` for numeric text
  input with step buttons.
- Use `gpui_form_collection::slider::Slider` for continuous numeric values.
- Use `gpui_form_collection::color_picker::ColorPicker` for color selection.
- Use `gpui_form_collection::date_picker::DatePicker` for single-date editing.
- Use `gpui_form_collection::date_picker::DateRangePicker` for date-range editing.
- Use `gpui_form_component::date_picker::DatePicker` or
  `gpui_form_component::date_picker::DateRangePicker` when the localized
  runtime date picker should be used directly as the form shape; enable
  `gpui-form-component`'s `component-shape` feature.
- Use `gpui_form_component::file_picker::FilePicker` for native file/directory
  selection; enable `gpui-form-component`'s `component-shape` feature.
- Use `gpui_form_collection::otp_input::OtpInput::<_>` for OTP inputs.
- Use `gpui_form_component::infinite_select::InfiniteSelect::<_>` for
  nested/cascading enum trees; derive `InfiniteSelect`. Use
  `InfiniteSelect::<_>.searchable(true)` when search should be enabled.
  Use a custom `GpuiComponentShape` wrapper when deeper runtime options are
  needed.
- Define a custom component shape around `gpui_form_component::date_picker` or
  `gpui_form_component::file_picker` only when the ready-made shape needs
  non-default runtime construction or rendering metadata.
- Use `#[gpui_form(component(my::Shape))]` when the app owns the state/widget contract.
- Treat `component(...)` as the only component field intent in
  `#[gpui_form(...)]`; runtime construction uses
  `GpuiComponentShape::new`.
- Component shapes own the default value-storage policy for non-optional
  fields. Implement `GpuiFormComponentShapePolicy` with `DirectValueStorage`
  when the reusable shape can synthesize default values. Required shape-backed values are reported
  by generated `validate()` and by fallible holder-to-model conversion, while
  default-synthesizing policies expose `holder.into_original()`.
- Holders for forms with skipped source fields expose
  `holder.present_fields()` as a typed snapshot for debug/preview formatting;
  JSON or text formatting belongs in the app or generator layer.

## MCP Submit Pattern

Enable the facade `mcp` feature only when forms should be exposed through MCP
tools. MCP calls use structured JSON arguments; they do not use `clap`, shell
argument parsing, or GPUI button callbacks. Exposed forms must opt in with
`#[gpui_form(mcp)]`, must not use `#[gpui_form(no_inventory)]`, and must not be
generic.

```rust
use gpui_form::GpuiForm;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, GpuiForm, Serialize)]
#[gpui_form(mcp)]
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

For skipped-field forms that need application-owned context, use the generated
`*FormValueHolder` as the first parameter; `#[gpui_form::mcp_submit]` infers
holder submission from that parameter type. Handlers must be synchronous or
async and return `Result<T, E>`, where `T: serde::Serialize +
gpui_form::mcp::McpJsonSchema`. Prefer a typed response struct or newtype that
derives `gpui_form::mcp::McpJsonSchema` for precise output schemas, or return
`gpui_form::mcp::McpObject` for dynamic object-shaped JSON.
For submit flows with shared runtime state, put `context(MyContext)` in the
form's struct-level `gpui_form(mcp(...))` options, optionally add
`response(MyResponse)` for a precise output schema, and either implement
`gpui_form::mcp::McpContextSubmit<MyContext>` for the source model or add
`submit(path::to::async_fn)` so the derive emits that impl. Generated submit
impls can also use `map_response(path::to::fn)` to convert a raw handler
response into the published response type, and `error(MyError)` to override
the default `String` error type. Use `submitter(path::to::Trait)` instead
when a visible trait supplies associated `Context`, `Response`, and `Error`
types plus `submit_with_context(self, context)`. Pair submitters with
`map_response(path::to::fn)` when the submitter returns a raw application
response that should be converted into the published MCP response. Then call
`gpui_form::mcp::register_context_submitters(&mut server, context)?` or
`register_context_submitters_with_editor_options(...)`. The derive inventories
those forms, so one registration call wires every matching context-backed
submit tool plus `*_edit_submit`; the context type must be
`Clone + Send + Sync + 'static`. If `response(...)` is omitted, the generated
registration uses `McpObject`. Use `register_context_submitters_strict(...)`
or `register_context_submitters_strict_with_editor_options(...)` when zero
matching context registrations should fail setup instead of succeeding as a
no-op; the strict variants return a registration report with the matched
context registration count.
Use `gpui_form::mcp::server()?` for the default generated server and
`gpui_form::mcp::server_named(name, version)?` when application-owned server
metadata is needed. Generated servers register `#[gpui_form::mcp_submit]`
submit handlers plus editable holder-session tools for every
`#[gpui_form(mcp)]` form. Forms that also have a submit handler get
`*_edit_submit`, so agents can submit a valid edit session by `session_id`.
Use `register_with_options(&mut server, McpFormRegistrationOptions::submit_only())`
to publish only generated submit handlers,
`McpFormRegistrationOptions::editor_only()` to publish only editable holder
sessions, or `McpFormRegistrationOptions::metadata_only()` to inspect selected
tool definitions and skipped-tool report without mutating the server.
`tool_definitions_with_options(...)` returns the profile-selected definitions
directly. `register()` remains equivalent to `McpFormRegistrationOptions::all()`.
Generated servers also advertise `resources` and publish JSON resources for
every registered form at `gpui-form://forms/{tool_name}/descriptor`,
`gpui-form://forms/{tool_name}/schema`, and
`gpui-form://forms/{tool_name}/examples`. The descriptor resource includes
form/tool metadata, resource links, field labels, descriptions, examples,
required/default/storage metadata, component MCP input metadata, validation
rules, and per-field schemas; the schema resource contains the submit input
schema; the examples resource groups field examples. `metadata_only()` does not
register resources. Use `form_resource_uris`, `form_descriptor_resource_value`,
`form_schema_resource_value`, and `form_examples_resource_value` in tests or
manual integrations that need the same resource contract without serving an
MCP transport.
Prompt templates are opt-in: use
`register_prompt_templates(&mut server)?` for inventory-discovered forms,
`register_context_submitter_prompt_templates::<Context>(...)` for
context-backed submitters, or `register_form_prompt_templates::<Form>(...)`
for one form. Generated prompt names are `fill_{tool_name}_form`,
`repair_{tool_name}_form`, and `submit_{tool_name}_form`; the text references
the generated descriptor/schema/examples resources and matching edit tools
when iterative repair is useful.
Use `gpui_form::mcp::builder()` or
`builder_named(name, version)` when deferred builder setup is needed.
Use `gpui_form::mcp::serve_stdio_blocking()` for the default stdio server.
Use `McpServer::builder(name, version)` when composing forms with tables or
other MCP integrations, and add generated handlers and editors with
`.register(gpui_form::mcp::register)`. Chain
`.register(gpui_form::mcp::register_prompt_templates)` when the server should
also expose generated prompt templates.
Register manual handlers with
`gpui_form::mcp::form::<Form>(&mut server).model(handler)?` or
`.holder(handler)?` for handlers returning `Result<T, E>`.
Use `.model_with_editor(handler)?`, `.model_async_with_editor(handler)?`,
`.holder_with_editor(handler)?`, or `.holder_async_with_editor(handler)?` when
the same form should publish submit, editable holder-session tools, and
`*_edit_submit` for submitting the current edit session through the same
handler.
Use the matching `*_with_editor_options` helpers with
`McpFormEditorOptions` when a server needs a custom session cap.
Editor `open`, `read`, `patch`, and `validate` responses include
the session `revision`, agent-supplied `values`, missing required fields,
validation `errors`, `valid`, `fields` metadata with per-field schemas, field
labels, descriptions, examples, current `has_value`/`value` state, and
field-level `errors`, plus
`submit_arguments` that can be passed to the submit tool.
Published snapshot schemas keep `fields[]` typed with a per-field `oneOf`.
`*_edit_list` returns all active sessions for that form using the same snapshot
shape plus `session_limit`, `session_idle_timeout_ms`, and `session_count`.
`*_edit_close` accepts optional `expected_revision` before closing one session.
`*_edit_close_all` closes every active session for that form and returns
`closed_count` plus the closed `session_ids`.
Editors retain at most `DEFAULT_EDITOR_SESSION_LIMIT` active sessions per form
by default and expire idle sessions after
`DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT`; opening another session evicts the oldest
session when the cap is exceeded. Use
`.editor_with_options(McpFormEditorOptions::default().with_session_limit(n).with_session_idle_timeout(duration))`
or
`McpFormEditorOptions::default().without_session_limit().without_session_idle_timeout()`
to tune that policy.
`edit_open` and `edit_list` report a `cleanup` object with expired and evicted
session IDs so clients can drop local handles that the server removed while
servicing the request.
`edit_patch` accepts a `session_id`, optional
`values` object, optional `clear` field-name array, and optional
`replace = true` to replace the whole pending value set instead of merging.
It also accepts optional `expected_revision` to reject stale client updates;
bulk patches decode by rebuilding a cloned holder before commit, so failed
patches do not partially mutate the session.
Handler-backed editors, including generated `#[gpui_form::mcp_submit]`
handlers, also register `*_edit_submit`, which accepts a `session_id`,
optional `expected_revision`, and optional `close_on_success = false`, validates
the current session, and calls the same submit handler without copying
`submit_arguments` into a second tool call. Successful edit submits close the
session by default only when the session has not advanced while the handler was
running; validation, stale revision, handler failure, or concurrent patch cases
leave it open for repair or retry.
Generated editor tool definitions publish MCP annotations: `edit_list`,
`edit_read`, and `edit_validate` are read-only and idempotent, `edit_open` is a
non-destructive session mutation, `edit_patch`, `edit_close`, and
`edit_close_all` are destructive session mutations, and `edit_submit` is a
destructive open-world submit call.
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
Use struct-level `#[gpui_form(mcp(...))]` with `name`, `title`, `description`,
`read_only`, `destructive`, `idempotent`, `open_world`, repeated `icon(...)`,
`task_support = "forbidden" | "optional" | "required"`, `context(Type)`, and
optional `response(Type)`, `error(Type)`, `submit(path)`, and
`map_response(path)`, or `submitter(TraitPath)` with optional
`map_response(path)` when generated MCP tools need application-owned metadata,
MCP tool annotation hints, MCP tool icons, execution task support,
context-submit inventory, generated context submit impls, trait-backed context
submitters, raw response mapping, or precise response
schemas. Direct submit tools default to destructive open-world annotations
unless overridden. If `description` is omitted, the derive uses the form type's
Rust doc comment.
`read_only = true` and `destructive = true` cannot be combined.
Use field-level `#[gpui_form(label = "...")]`,
`#[gpui_form(description = "...")]`, and `#[gpui_form(example = "...")]` for
MCP field metadata. Field descriptions infer from `///` rustdoc when not
overridden, labels default from the field name, and examples may be repeated.
Registration reports setup errors such as duplicate tool names or generated
resource URIs.

MCP input schemas use the field type's `McpToolValue` schema. Component-backed
fields also attach value-specific `<Shape as ComponentShapeFor<Field>>::MCP_INPUT`
metadata when it is available.
When Koruma validation is enabled, generated field schemas attach
`x-gpuiFormValidation` rule metadata. Literal `LenValidation`,
`RangeValidation`, and `NonEmptyValidation` arguments are also reflected as
JSON Schema hints such as `minLength`, `maxLength`, `minimum`,
`maximum`, `exclusiveMinimum`, `exclusiveMaximum`, `minItems`, and `maxItems`
when the field schema type is unambiguous.
Generated MCP validation errors use gpui-form's field rule metadata first and
fall back to Koruma's generic `ValidationIssues` output when no field-specific
rule extractor is available.
`component_shape_gpui` infers common MCP metadata from unambiguous declared
values such as `String`, booleans, numbers, dates, `Vec<T>`, set-like
primitive collections, fixed arrays, `gpui_form::mcp::McpRange<T>`, or
`(Option<T>, Option<T>)` ranges. `Vec<T>` and fixed arrays publish list
metadata; set-like collections publish set metadata. Inferred metadata is
attached to generated `ComponentShapeFor<Value>` impls, so multi-value shapes
can expose distinct MCP input shapes per supported value. For custom field
value types, generated descriptors require `gpui_form::mcp::McpToolValue`; the
blanket implementation covers `Deserialize` types that implement or derive
`gpui_form::mcp::McpJsonSchema`; use `gpui_form::mcp::McpAny` when a typed
field intentionally accepts unconstrained JSON. Type aliases inherit the
underlying schema. Fixed tuples with 1 to 4 elements publish exact array
schemas. Generic or custom component shapes can declare `mcp_input = string`,
`mcp_input = object`, or another `McpInput` expression when the shape knows its
model-facing MCP input better than the value type can be inferred. App-owned
object structs, tuple or named transparent newtypes, and fieldless enums can
derive `gpui_form::mcp::McpJsonSchema` directly. In crates that also depend on
another MCP facade such as `gpui-table`, add
`#[mcp(crate = gpui_form::mcp)]` to custom `McpJsonSchema` or `McpToolInput`
derives so generated schema impls target the gpui-form facade. The derive
follows serde deserialize names, records field aliases in `x-mcpAliases`,
includes enum aliases, skips deserialization-skipped fields, rejects flattened
fields, and treats serde-defaulted fields as not required. Custom top-level MCP
inputs can derive `gpui_form::mcp::McpToolInput`; that derive also implements
`McpJsonSchema`, so object inputs can be reused as field values.

## Generated Names

For a source struct named `UserProfile`, expect generated types named:

```rust
UserProfileFormFields
UserProfileFormComponents
UserProfileFormValueHolder
```

Use the generated value holder for editable form data, defaults, and conversion
back into the original model.

## Select Pattern

```rust
use gpui_form_collection_derive::SelectItem;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    France,
}
```

`SelectItem` uses enum variant names as fallback labels by default. Add
`#[select_item(display)]` only when `title()` should call `Display`, or
`#[select_item(fluent)]` when the enum derives `EsFluent` and the app will
handle localized labels outside the `SelectItem::title()` call.

## Infinite Select Pattern

```rust
use gpui_form::GpuiForm;
use gpui_form_component::InfiniteSelect;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, InfiniteSelect, PartialEq)]
pub enum City {
    #[default]
    Paris,
    Lyon,
}

#[derive(Clone, Debug, EnumIter, InfiniteSelect, PartialEq)]
pub enum Country {
    France(City),
}

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct LocationForm {
    #[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>))]
    pub location: Country,
}
```

Helper state is available from `gpui_form_component::infinite_select`.

## Type Conversion Pattern

Use intent-scoped `value(type = ..., from_source = ..., into_source = ...)`
when the UI edits a different type than the model stores:

```rust
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
    pub birth_date: Option<Timestamp>,
}
```

This is useful for dates, paths, numeric newtypes, and other domain-specific
wrappers. Replace `into_source` with `try_into_source` when reconstruction can
fail. For Koruma newtypes, `value(koruma_newtype)` expands to the public
`NewtypeValue` / `NewtypeTryFromInner` trait surface and works with private
wrapper fields.

## Component Shape Patterns

Derive on the rendered component. The derive infers a same-module backing state
named after the component, such as `TagsInputState`; declare `state = ...` only
when the backing state uses a different name or path:

```rust
use gpui_form::GpuiForm;
use component_shape_gpui::GpuiComponentShape;

#[derive(GpuiComponentShape)]
#[gpui_component_shape(
    value = Vec<String>,
    field_suffix = "input"
)]
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

`new` accepts a constructor path or closure and is called with `(window, cx)`;
if omitted, the derive calls `<State>::new(window, cx)`.
Crates that use `#[derive(GpuiComponentShape)]`, `component_shape!`, or direct
runtime value-binding impls should depend on `gpui-form-runtime` directly
because those lower-level surfaces emit direct runtime paths.

Or declare a reusable shape:

```rust
component_shape_gpui::component_shape! {
    pub struct EmailInputShape {
        state = gpui_component::input::InputState;
        component = gpui_component::input::Input;
        value = String;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for EmailInputShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}
```

`component_shape_gpui::component_shape!` uses semicolons between options.
Use `value = ...` or `values(...)` to publish supported form-side value types
when the shape is not value-bound.
Use `mcp_input = string`, another common constructor shorthand, or any
`McpInput` expression when a generic/custom shape knows its model-facing MCP
input better than type inference can.
Manual
`GpuiComponentShapeFor<Value>` impls are rejected inside `component_shape!`.
Implement `GpuiFormComponentShapePolicy` when the reusable shape should be used
by `#[derive(GpuiForm)]`. Put `GpuiComponentValueBinding<T>` impls inside the
macro block when the wrapper shape owns reusable synchronization; when no
explicit `value = ...` or `values(...)` metadata is present, the nested binding
impl publishes both compatible value metadata and shape-level value-binding
metadata. `component = ...` must be a path-like type, `mcp_input = ...` accepts
common constructor shorthands or a `McpInput` expression, and
`field_suffix = "..."` must be a non-empty ASCII identifier suffix.

Do not hand-write `GpuiComponentShape` for `#[gpui_form(component(...))]`
fields. `#[derive(GpuiForm)]` requires the `DeclaredGpuiComponentShape` marker
emitted by `#[derive(GpuiComponentShape)]` or `component_shape!`.
