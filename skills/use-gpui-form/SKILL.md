---
name: use-gpui-form
description: "Use when Codex needs to build user-facing application forms with gpui-form, including adding #[derive(GpuiForm)] to app structs, choosing existing gpui_form component shapes, using generated form fields/components/value holders, and wiring SelectItem or InfiniteSelect enums."
---

# Use GPUI Form

## Scope Boundary

Treat this skill as a hosted public-usage guide for `gpui-form` consumers. Use
it only for user-facing application workflows: deriving forms on app models,
choosing component attributes, using generated form state, wiring select and
infinite-select enums, and applying existing built-in, collection, or
component-owned shapes.

For custom app-owned components, external state wrappers, reusable
`GpuiComponentShape` implementations, or shape-level value bindings, use
`use-gpui-form-component-shapes`.

This reusable skill does not cover crate internals, release workflow,
repository maintenance, or contributor-only architecture work. Use
project-specific contributor documentation for those tasks.

## Core Workflow

Start from the user-facing facade for `GpuiForm`, then import component runtime
helpers and component-specific derives explicitly:

1. Identify the application struct or enum that should drive the form.
2. Check the app's existing `Cargo.toml` and form code for local patterns.
3. Use the facade and component derive crates in application code:

   ```rust
   use gpui_form::GpuiForm;
   use gpui_form_collection_derive::SelectItem;
   ```

4. Add `GpuiForm` to a normal Rust struct and give every field one explicit
   intent: component, `hidden`, or `skip`.
5. Use generated types named from the source struct, such as
   `UserProfileFormFields`, `UserProfileFormComponents`, and
   `UserProfileFormValueHolder`.
6. Put the component shape in `#[gpui_form(component(...))]`, use
   `#[gpui_form(hidden)]` for value-holder-only fields,
   intent-scoped `default = ...` inside `component(...)` or `hidden(...)` for initial form values,
   `#[gpui_form(skip)]` for model fields that should not render as widgets, and
   `#[gpui_form(component(Shape, value(type = ..., from_source = ..., into_source = ...), default = ...))]`
   when the UI edits a form-side type that differs from the model field. Use
   `try_into_source = ...` when the reverse conversion returns `Result` with a
   debug-formatted error, and
   `value(koruma_newtype)` when editing the inner value of a struct-level
   `#[koruma(newtype)]` field. Do not use `Option<T>` in `value(type = ...)`;
   write the form-side base type `T` and let the source field optionality control
   holder storage. Do not combine
   `skip` with component or hidden intent on the same field. Text input
   prototyping parses non-`String` form-side types with `FromStr`.
7. Add `#[gpui_form(no_inventory)]` to generic form structs when the
   `inventory` feature is enabled; generic forms cannot register concrete
   prototyping metadata.
8. For MCP submit tools, enable `gpui-form`'s `mcp` feature, derive `serde`
   support for field value types, add `#[gpui_form(mcp)]` to a concrete
   non-`no_inventory` form type, and annotate application-owned handlers with
   `#[gpui_form::mcp_submit]`. Use a source-model first parameter for model
   handlers and a generated `*FormValueHolder` first parameter when
   skipped-field forms need application-owned context.
   For many forms sharing one runtime context, put
   `context(MyContext)` inside each struct's `gpui_form(mcp(...))`, optionally
   add `response(MyResponse)` for a precise output schema, implement
   `McpContextSubmit<MyContext>` for those source models or add
   `submit(path::to::async_fn)` so the derive emits the impl, and register them with
   `gpui_form::mcp::register_context_submitters(...)` or the
   `*_with_editor_options` variant. Omitted `response(...)` uses
   `gpui_form::mcp::McpObject` for dynamic object-shaped JSON. Use
   `map_response(path::to::fn)` when the async submit function returns a raw
   value that needs conversion, and `error(MyError)` for non-`String` errors.
   Use `submitter(path::to::Trait)` instead when a visible trait supplies
   associated `Context`, `Response`, and `Error` types plus
   `submit_with_context(self, context)`. Pair submitters with
   `map_response(path::to::fn)` when the submitter returns a raw application
   response that should be converted into the published MCP response.
   Handlers can be synchronous or async and must return `Result<T, E>`.
   Do not add CLI, shell parsing, or GPUI button-callback execution for MCP.
   MCP input schemas use the field type's `McpToolValue` schema. Component-backed
   fields also attach value-specific `<Shape as ComponentShapeFor<Field>>::MCP_INPUT`
   metadata when the component shape publishes it.
9. Use paths such as `gpui_form_component::date_picker`,
   `gpui_form_component::file_picker`, and
   `gpui_form_component::infinite_select` for helper state.
   Generated `GpuiForm` component fields use the facade path
   `gpui_form::runtime::shape`, so normal application crates do not add
   `gpui-form-runtime` just because a field uses a component shape.
   Use `gpui_form_component::file_picker::FilePicker` as a ready-made form
   shape when `gpui-form-component` has its `component-shape` feature enabled;
   generated fields store `FilePickerState` as the backing entity state.
   `gpui_form_component::date_picker::DatePicker` and
   `gpui_form_component::date_picker::DateRangePicker` are also ready-made
   localized date form shapes under the same feature.
   Directly-reusable collection shapes are also available as
   `gpui_form_collection::date_picker::DatePicker`,
   `gpui_form_collection::date_picker::DateRangePicker`,
   `gpui_form_collection::combobox::Combobox::<Item>`,
   `gpui_form_collection::number_input::NumberInput::<_>`,
   `gpui_form_collection::slider::Slider`,
   `gpui_form_collection::color_picker::ColorPicker`, and
   `gpui_form_collection::otp_input::OtpInput::<_>`.
   If the app needs to create a new reusable component shape or wrap an
   external component/state pair, switch to `use-gpui-form-component-shapes`.

## Reference Selection

Load only the reference needed for the task:

- `references/api-map.md`: installation shape, supported component syntax, and user-facing usage patterns.

Prefer current public docs or source examples over memory when details matter.

## Implementation Rules

Derive application forms from app-owned data types:

```rust
use gpui_form::GpuiForm;
use gpui_form_collection_derive::SelectItem;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    France,
}

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub username: Option<String>,

    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub age: Option<u32>,

    #[gpui_form(
        component(gpui_form_collection::select::Select::<_>::searchable(true)),
        default = Country::France
    )]
    pub country: Country,
}
```

Common patterns:

- For selects, derive `SelectItem` from `gpui-form-collection-derive` on enum-like values and `EnumIter` when the app needs iteration-backed choices. `SelectItem` uses variant-name fallback labels by default and only needs `Display` with `#[select_item(display)]`. Use `Select::<_>::searchable(true)` when the field should construct the built-in select with search enabled; use `Select::<_>::from(SelectArgs::builder().searchable(true).build())` when passing a completed Bon-built select configuration.
- For cascading or nested selects, derive `InfiniteSelect` from `gpui-form-component` with its `derive` feature and `PartialEq` on the enum tree. Enable `gpui-form-component`'s `component-shape` feature when using `#[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>))]` directly. Use `InfiniteSelect::<_>::searchable(true)` when the cascading selects should construct with search enabled.
- Component shapes own the default value-storage policy for non-optional fields. Use plain built-in default-synthesizing shapes such as `Input::<_>`, `Select::<_>`, `Combobox::<Item>`, `Checkbox`, `Switch`, `NumberInput::<_>`, `Slider`, `OtpInput::<_>`, `FilePicker`, and `InfiniteSelect::<_>`. Date picker and color picker shapes can back optional fields; for a required model field that should start empty, omit `default = ...` and use the fallible holder-to-model path. Required shape-backed values are visible to generated `validate()` as well as fallible holder-to-model conversion.
- For Koruma validation, add `#[gpui_form(koruma)]` or `#[gpui_form(koruma(fluent))]` to the form struct and write field validators with Koruma's direct syntax, such as `#[koruma(NonEmptyValidation::<_>)]` or `#[koruma(RangeValidation::<_>::min(18).max(120))]`. Use Koruma target selectors such as `full(...)` when a validator must receive the full optional value.
- Koruma-backed MCP forms publish validation rule metadata in generated field
  schemas through `x-gpuiFormValidation`. Literal `LenValidation`,
  `RangeValidation`, and `NonEmptyValidation` arguments become JSON Schema
  hints when the field schema type is unambiguous. Validation failures expose
  structured issue objects in `structured_content.error.details`, editor
  session `errors`, and field-level `errors`; generated MCP validation uses
  gpui-form's field rule metadata first and falls back to Koruma's generic
  `ValidationIssues` output when no field-specific extractor is available.
- Convert generated holders with `holder.try_into_original()` when conversion
  can fail, when a field uses `try_into_source` or `value(koruma_newtype)`, or
  when a required shape-backed field has no declared default,
  `holder.into_original()` when it is statically infallible, or
  `holder.into_original(skipped_value, ...)` when the source model has skipped
  fields that the form cannot edit. For skipped-field debug UI, format
  `holder.present_fields()` outside the generated holder instead of expecting a
  JSON helper.
- For MCP submit integration, keep GPUI as a presentation surface over the same
  holder and submit handler. MCP tool calls should populate the generated value
  holder from structured JSON, run generated validation, and call an
  application-owned handler registered through `#[gpui_form::mcp_submit]`.
  The first parameter chooses the mode through generated `McpSubmitArgument`
  implementations: source model for model submission, generated
  `*FormValueHolder` for holder submission.
  Exposed forms must opt in with `#[gpui_form(mcp)]`, must not use
  `#[gpui_form(no_inventory)]`, and must not be generic.
  Handlers can be synchronous or async and must return `Result<T, E>`.
  Use `gpui_form::mcp::server()?` for the default generated server and
  `gpui_form::mcp::server_named(name, version)?` when application-owned server
  metadata is needed. Use `gpui_form::mcp::builder()` or
  `builder_named(name, version)` when deferred builder setup is needed.
  Use `gpui_form::mcp::serve_stdio_blocking()` for the default stdio server.
  Use `McpServer::builder(name, version)` when composing forms with tables or
  other MCP integrations, and add generated handlers with
  `.register(gpui_form::mcp::register)`. Add
  `.register(gpui_form::mcp::register_prompt_templates)` when the server
  should also advertise generated fill, repair, and submit prompt templates.
  Use `register_with_options(&mut server, McpFormRegistrationOptions::submit_only())`
  for quiet submit-only tool lists, `McpFormRegistrationOptions::editor_only()`
  for editable holder sessions without submit handlers, or
  `McpFormRegistrationOptions::metadata_only()`/`tool_definitions_with_options(...)`
  to inspect generated definitions without mutating a server.
  Generated MCP servers also advertise `resources` and publish JSON resources
  for each registered form at
  `gpui-form://forms/{tool_name}/descriptor`,
  `gpui-form://forms/{tool_name}/schema`, and
  `gpui-form://forms/{tool_name}/examples`. The descriptor resource includes
  field labels, descriptions, examples, storage/default metadata, component
  MCP input metadata, validation rules, and per-field schemas; the schema
  resource contains the submit input schema. `metadata_only()` does not
  register tools or resources.
  Prompt templates are opt-in: use `register_prompt_templates(&mut server)?`
  for inventory-discovered forms,
  `register_context_submitter_prompt_templates::<Context>(...)` for
  context-backed submitters, or `register_form_prompt_templates::<Form>(...)`
  for one form. Generated names are `fill_{tool_name}_form`,
  `repair_{tool_name}_form`, and `submit_{tool_name}_form`, and the prompt
  text references the descriptor/schema/examples resources plus edit tools
  when iterative repair is useful.
  Register manual handlers with
  `gpui_form::mcp::form::<Form>(&mut server).model(handler)?` or
  `.holder(handler)?` for handlers returning `Result<T, E>`.
  For context-backed fleets of form submit tools, prefer struct-level
  `context(MyContext)` MCP options plus `McpContextSubmit<MyContext>`
  implementations or generated impls from `submit(path::to::async_fn)`,
  or `submitter(path::to::Trait)` when one visible trait should supply the
  context, response, error, and submit method, adding `map_response(path)` when
  that trait returns a raw response and `response(MyResponse)` only when precise
  response schemas matter, then call
  `gpui_form::mcp::register_context_submitters(&mut server, context)?` once.
  Use `register_context_submitters_strict(...)` when zero matching context
  registrations should be a setup error.
  Register headless field-editing tools with
  `gpui_form::mcp::form::<Form>(&mut server).editor()?`. The generated editor
  tools open a bounded value-holder session, bulk patch or clear fields through
  the same component-shape-aware decoder used by submit calls, expose
  `revision`, `session_limit`, `session_idle_timeout_ms`, agent-supplied
  values, and `submit_arguments`, validate, expire idle sessions, and close the
  session. Use `McpFormEditorOptions` or the `*_with_editor_options` submit
  helpers when a server needs a custom session cap or idle timeout. Treat this
  as holder editing only; live GPUI widget mutation still needs an app-owned
  bridge from the holder/session into GPUI entities.
  Use struct-level `#[gpui_form(mcp(name = "...", title = "...", description = "..."))]`
  when generated MCP tools need application-owned names or descriptions. Add
  `context(Type)` there when the form should emit context-submit inventory,
  `submit(path)` when the derive should generate `McpContextSubmit`, and
  `response(Type)` only to override the default `McpObject` response. Use
  `submitter(TraitPath)` instead of those context-submit wiring options when a
  visible trait supplies the associated context, response, error, and submit
  method, and pair it with `map_response(path)` when the trait returns a raw
  application response.
  Direct submit tools default to destructive open-world MCP annotations unless
  overridden. Add `icon(src = "...", mime_type = "...", size = "...", theme = "light" | "dark")`
  entries for MCP tool icons, and
  `task_support = "forbidden" | "optional" | "required"` for MCP execution task
  support.
  If `description` is omitted, the derive uses the form type's Rust doc
  comment.
  Use field-level `#[gpui_form(label = "...")]`,
  `#[gpui_form(description = "...")]`, and `#[gpui_form(example = "...")]`
  when MCP schemas and editor snapshots need form-author labels,
  descriptions, or examples; field descriptions infer from `///` rustdoc when
  not overridden.
  Registration reports setup errors such as duplicate tool names or generated
  resource URIs.
  Component-backed fields use value-specific
  `<Shape as ComponentShapeFor<Field>>::MCP_INPUT` metadata in generated
  schemas when available, then fall back to the field type's schema.
  Generic or custom component shapes can declare `mcp_input = string`,
  `mcp_input = object`, or another `McpInput` expression when the shape knows
  its model-facing MCP input better than the value type can be inferred.
  Custom field value types exposed through MCP should implement
  `gpui_form::mcp::McpToolValue`; this is automatic for `Deserialize` types
  that implement or derive `gpui_form::mcp::McpJsonSchema`; use
  `gpui_form::mcp::McpAny` when a typed field intentionally accepts
  unconstrained JSON. Aliases inherit their target type schema, fixed tuples
  with 1 to 4 elements publish exact array schemas, and tuple or named
  transparent newtypes, named structs, or fieldless enums can derive
  `McpJsonSchema` directly through `gpui_form::mcp`. The
  derive follows serde deserialize names, records field aliases in
  `x-mcpAliases`, includes enum aliases, skips deserialization-skipped fields,
  rejects flattened fields, and treats serde-defaulted fields as not required.
  Custom top-level MCP inputs can derive `gpui_form::mcp::McpToolInput`; that
  derive also implements `McpJsonSchema`, so object inputs can be reused as
  field values. Use `gpui_form::mcp::McpRange<T>` for typed
  `{ "min": ..., "max": ... }` range arguments.
- `Combobox::<Item>` treats an empty selection as `FormValueChange::Clear`; optional fields clear to `None`, while non-optional `Vec<Item>` fields reset to their intent-scoped `default = ...` when present, otherwise `Vec::default()`.
- For app-owned widgets, external component/state wrappers, custom search/depth options, reusable `GpuiComponentShape` implementations, or shape-level value bindings, use `use-gpui-form-component-shapes`.
- Collection and component-owned shapes publish prototyping suffixes such as `input`, `select`, `combobox`, `checkbox`, `switch`, `number_input`, `slider`, `color_picker`, `date_picker`, `date_range_picker`,
  `file_picker`, `infinite_select`, and `otp_input`. Generated `FormFields` members and
  `FormComponents` constructors use the source field name. Derive-emitted inventory uses
  shape-level `field_suffix = "..."` metadata when available, or the shape type fallback otherwise,
  for prototyping-specific DOM IDs, event handlers, and helper names. Shape-level suffixes must be
  non-empty ASCII identifier suffixes. Direct `ComponentPrototyping::field_suffix(...)` calls
  validate the same suffix contract.
- Inventory prototyping fails fast when a shape-backed field is missing render,
  value-binding, shape path, or default-storage metadata. Fix the component
  shape metadata instead of relying on placeholder generated UI.
- Format written inventory-prototyping scaffolds with `rustfmt`; the workspace `examples/prototyping` generator does this before reporting completion.
- Storybook-style GPUI scaffolds that localize generated labels or messages should call `gpui_es_fluent::localize_label` and `gpui_es_fluent::localize_message` with the active `gpui::App` context.
- Keep consumer code focused on app models, form state, rendering, and app-owned components.
