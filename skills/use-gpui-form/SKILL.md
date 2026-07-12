---
name: use-gpui-form
description: "Use when Codex needs to build or expose user-facing application forms with gpui-form, including #[derive(GpuiForm)], field intents and conversions, generated fields/components/value holders, built-in component shapes, SelectItem or InfiniteSelect enums, Koruma validation, inventory prototyping, and MCP submit or editor tools."
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
   `skip` with component or hidden intent on the same field. `Input::<_>`,
   `NumberInput::<_>`, and `OtpInput::<_>` parse non-`String` form-side types
   with `FromStr` in generated prototyping code.
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
        component(gpui_form_collection::select::Select::<_>.searchable(true)),
        default = Country::France
    )]
    pub country: Country,
}
```

Common patterns:

- For text-like value objects that need custom parsing or display formatting,
  use `gpui_form_collection::input::ParsedInput::<_, Config>` and implement
  `ParsedInputConfig<T>` with the parser, formatter, placeholder,
  empty-as-clear policy, and optional widget validation.
- For selects, derive `SelectItem` from `gpui-form-collection-derive` on enum-like values and `EnumIter` when the app needs iteration-backed choices. `SelectItem` uses variant-name fallback labels by default and only needs `Display` with `#[select_item(display)]`. Use `Select::<_>.searchable(true)` when the field should construct the built-in select with search enabled; use `Select::<_>.from(SelectArgs::builder().searchable(true).build())` when passing a completed Bon-built select configuration.
- For cascading or nested selects, derive `InfiniteSelect` from `gpui-form-component` with its `derive` feature and `PartialEq` on the enum tree. Enable `gpui-form-component`'s `component-shape` feature when using `#[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>))]` directly. Use `InfiniteSelect::<_>.searchable(true)` when the cascading selects should construct with search enabled.
- Component shapes own the default value-storage policy for non-optional fields. Use plain built-in default-synthesizing shapes such as `Input::<_>`, `Select::<_>`, `Combobox::<Item>`, `Checkbox`, `Switch`, `NumberInput::<_>`, `Slider`, `OtpInput::<_>`, `FilePicker`, and `InfiniteSelect::<_>`. Date picker and color picker shapes can back optional fields; for a required model field that should start empty, omit `default = ...` and use the fallible holder-to-model path. Required shape-backed values are visible to generated `validate()` as well as fallible holder-to-model conversion.
- For Koruma validation, add `#[gpui_form(koruma)]` or `#[gpui_form(koruma(fluent))]` to the form struct and write field validators with Koruma's direct syntax, such as `#[koruma(NonEmptyValidation::<_>)]` or `#[koruma(RangeValidation::<_>.min(18).max(120))]`. Use Koruma target selectors such as `full(...)` when a validator must receive the full optional value.
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
  generated holder and application-owned submit handler. Opt concrete,
  inventory-backed forms in with `#[gpui_form(mcp)]`; use a source-model first
  handler parameter for model submission or a generated `*FormValueHolder`
  parameter when skipped fields require application-owned context. Register
  generated or manual submitters through `gpui_form::mcp`, and use generated
  editor tools only for headless holder sessions; live GPUI entity mutation
  still needs an app-owned bridge.
- Generated MCP field schemas always use the field type's `McpToolValue`
  schema. Component-backed fields additionally attach value-specific
  `<Shape as ComponentShapeFor<Field>>::MCP_INPUT` metadata when the shape
  publishes it. Use `McpJsonSchema`, `McpToolInput`, `McpAny`, and `McpRange<T>`
  through the `gpui_form::mcp` facade for custom typed wire contracts.
- Read `references/api-map.md` for complete MCP registration profiles,
  context submitters, editor session options, resources, prompt templates,
  annotations, validation metadata, and schema patterns.
- `Combobox::<Item>` treats an empty selection as `ValueChange::Clear`;
  optional fields clear to `None`, while non-optional `Vec<Item>` fields reset
  to their intent-scoped `default = ...` when present, otherwise
  `Vec::default()`.
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
