# gpui-form-derive

Procedural macros behind the `gpui-form` ecosystem.

Most users should depend on [`gpui-form`](../gpui-form/README.md) and derive
from the facade crate. Use this crate directly when you want only the
`GpuiForm` proc macro.

## What This Crate Provides

- `#[derive(GpuiForm)]`

GPUI component shape derives and helper macros live in
[`component-shape-gpui`](https://github.com/stayhydated/component-shape/tree/master/crates/component-shape-gpui).

`#[derive(InfiniteSelect)]` does not live in this crate. It is provided by
[`gpui-form-component-derive`](../gpui-form-component-derive/README.md) and by
`gpui-form-component` when that crate's `derive` feature is enabled.

`#[derive(SelectItem)]` is implemented in
[`gpui-form-collection-derive`](../gpui-form-collection-derive/README.md).

## `#[derive(GpuiForm)]`

Turns a struct into typed form state plus helper types for editing and
submission.

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub username: Option<String>,

    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub age: Option<u32>,
}
```

Supported component forms:

- `#[gpui_form(component(my::Shape))]`
- `#[gpui_form(component(gpui_form_collection::input::Input::<_>))]`
- `#[gpui_form(component(gpui_form_collection::select::Select::<_>))]`
- `#[gpui_form(component(gpui_form_collection::select::Select::<_>::searchable(true)))]`
- `#[gpui_form(component(gpui_form_collection::select::Select::<_>::from(SelectArgs::builder().searchable(true).build())))]`
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

The `gpui_form_component` shape examples require that crate's
`component-shape` feature. Infinite-select field types also need the
`InfiniteSelect` derive, available through `gpui-form-component`'s `derive`
feature or by depending on `gpui-form-component-derive` directly.

The `component(...)` value starts with a Rust shape path. Plain paths delegate
runtime construction to `GpuiComponentShape::new`; configured expressions such
as `Select::<_>::searchable(true)` or
`Select::<_>::from(SelectArgs::builder().searchable(true).build())` use the
expression as a `GpuiComponentShapeBuilder` for the same base shape. The shape
type must be declared with `component_shape_gpui::component_shape!` or
`#[derive(component_shape_gpui::GpuiComponentShape)]`; hand-written
`GpuiComponentShape` impls are not accepted by `#[derive(GpuiForm)]`. Put render
component metadata and prototyping suffix metadata on the GPUI shape
declaration, and put form storage metadata on a
`gpui_form_runtime::shape::GpuiFormComponentShapePolicy` impl.

Supporting field attributes:

- `#[gpui_form(component(<shape>))]`
- `#[gpui_form(hidden)]`
- `#[gpui_form(component(<shape>, default = <expr>))]`
- `#[gpui_form(hidden(default = <expr>))]`
- `#[gpui_form(skip)]`
- `#[gpui_form(component(<shape>, value(type = <form_type>, from_source = <expr>, into_source = <expr>), default = <expr>))]`
- `#[gpui_form(hidden(value(type = <form_type>, from_source = <expr>, into_source = <expr>), default = <expr>))]`
- `#[gpui_form(label = "...")]`
- `#[gpui_form(description = "...")]`
- `#[gpui_form(example = "...")]`

Every non-empty-form field must choose exactly one intent: component, hidden,
or skipped. Use `hidden` for value-holder-only fields. Defaults and value
conversion options belong inside the component or hidden intent; `from_source`
maps source model values into form values, and `into_source` maps form values
back into source values. MCP field descriptions infer from `///` rustdoc when
no explicit `description = ...` is present; labels default from the field name,
and `example = ...` may be repeated.

Supporting struct attributes:

- `#[gpui_form(empty)]`
- `#[gpui_form(koruma)]`
- `#[gpui_form(koruma(fluent))]`
- `#[gpui_form(no_inventory)]`
- `#[gpui_form(mcp)]` or
  `#[gpui_form(mcp(...))]` with `name`, `title`, `description`, `read_only`,
  `destructive`, `idempotent`, `open_world`, repeated `icon(...)`,
  `task_support = "forbidden" | "optional" | "required"`, `context(Type)`,
  and optional `response(Type)`, `error(Type)`, `submit(path)`, and
  `map_response(path)` options

Behavior notes:

- reusable gpui-component-backed shapes live in `gpui-form-collection`
- collection components that are enum-driven (`Select`, `Combobox`) expect enum-like
  values that can populate a `gpui_component` list widget
- `gpui_form_component::infinite_select::InfiniteSelect::<_>` expects the field type
  to derive `gpui_form_component::InfiniteSelect`, which implements
  `gpui_form_component::infinite_select::InfiniteSelectValue`
- `default = ...` seeds the generated value holder
- component shapes own the default value-storage policy for non-optional
  fields; shapes that can synthesize a default value keep generated value-holder
  storage as `T`, while required shapes use `Option<T>` and fail conversion when
  missing. Generated `validate()` reports missing required shape values.
  Fallible holder-to-model paths expose `holder.try_into_original()`;
  infallible paths expose `holder.into_original()` and implement `From` when
  the derive can prove infallibility directly, such as optional fields, direct
  fields, or shape-backed fields with a declared field default. Shape-backed
  fields without a field default keep the checked `try_into_original()` path.
- set `GpuiFormComponentShapePolicy::ValueStoragePolicy` to
  `DirectValueStorage` on reusable shapes that can synthesize default values
- `ComponentShapeMetadata::CAPABILITIES` records whether the component shape
  implements `gpui_form_runtime::shape::GpuiComponentValueBinding<T>` for
  generated prototyping subscriptions; the adapter seeds component state with
  `seed_value_binding_state` and maps component events to `ValueChange<T>`
- generated `GpuiForm` component field code references
  `gpui_form::runtime::shape`; normal application crates do not need
  `gpui-form-runtime` directly just because a field uses a component shape
- the derive crate's `mcp` feature emits `gpui_form::mcp::McpForm` and
  `McpEditableForm` implementations plus MCP inventory registration beside
  `GpuiFormShape` only for forms that opt in with
  `#[gpui_form(mcp)]` (or `#[gpui_form(mcp(...))]`).
  Most users should enable this through
  `gpui-form = { features = ["mcp"] }`, which also enables the facade re-export
  and `gpui-form-mcp` integration crate.
  MCP submit forms must be concrete, cannot use `#[gpui_form(no_inventory)]`,
  and require the `mcp` feature when the attribute is present.
- for Koruma-backed MCP forms, the derive also emits structured validation
  issue extraction and attaches `x-gpuiFormValidation` rule metadata to field
  schemas. Literal `LenValidation`, `RangeValidation`, and
  `NonEmptyValidation` arguments become JSON Schema constraint hints when the
  field schema type is unambiguous.
- MCP argument decoding writes through generated value-holder storage and shape
  value-storage policies; it does not instantiate GPUI state or route through
  button callbacks. Skipped-field forms get holder submission support but no
  generated model-handler conversion. Component-backed field schemas use
  value-specific `<Shape as ComponentShapeFor<Field>>::MCP_INPUT` metadata when
  available. Struct-level
  `#[gpui_form(mcp(name = "...", title = "...", description = "..."))]`
  overrides generated MCP tool metadata; when `description` is omitted, the
  derive uses the form type's Rust doc comment. The same list accepts optional
  MCP tool annotation hints with `read_only = ...`, `destructive = ...`,
  `idempotent = ...`, and `open_world = ...`. Direct submit tools default to
  destructive open-world annotations unless overridden. Use repeated
  `icon(src = "...", mime_type = "...", size = "...", theme = "light" | "dark")`
  entries for MCP tool icons, and `task_support = "forbidden" | "optional" |
  "required"` for MCP tool execution task support.
  Adding `context(MyContext)` emits context-submit inventory for forms that
  implement `gpui_form::mcp::McpContextSubmit<MyContext>`. Add
  `submit(path::to::async_fn)` to have the derive generate that impl instead;
  the function is called with `(model, context)`. Add `map_response(path::to::fn)`
  when the async function returns a raw value that needs conversion into the
  published response, and `error(MyError)` when the generated impl should use a
  non-`String` error type. Add `response(MyResponse)` when the response should
  publish a precise schema; otherwise the registration uses
  `gpui_form::mcp::McpObject`. Use
  `gpui_form::mcp::register_context_submitters(...)` or the
  `*_with_editor_options` variant to register all matching context-backed
  submit tools and their `*_edit_submit` tools from one shared context value.
- the same opt-in emits a `gpui_form::mcp::McpEditableForm` implementation.
  Generated MCP servers register those editor tools by default, and the editor
  API decodes one field at a time through the same generated holder storage
  path for headless MCP edit sessions.
- the `#[gpui_form::mcp_submit]` attribute macro registers application-owned
  submit functions into MCP handler inventory. It infers model submission from
  the source model's generated `McpSubmitArgument` impl and holder submission
  from the generated `*FormValueHolder` impl (for skipped fields or app-owned
  context).
  Handlers must be synchronous or async and return `Result<T, E>`, where
  `T: serde::Serialize + gpui_form::mcp::McpJsonSchema`. Prefer a typed
  response struct or newtype whose schema declares an object root. The attribute
  macro resolves the facade crate
  path, so renamed `gpui-form` dependencies work like the derive output.
- `value(type = ..., from_source = ..., into_source = ...)` lets the generated
  holder edit a type that differs from the original model field. The `type = ...`
  option is the form-side base value type; use `type = T`, not
  `type = Option<T>`, even when the source field is optional.
- `gpui_form_collection::input::Input::<_>`,
  `gpui_form_collection::number_input::NumberInput::<_>`, and
  `gpui_form_collection::otp_input::OtpInput::<_>` prototyping code parse
  form-side non-`String` values with `FromStr` instead of assigning raw
  `String`s
- generic component expressions use `::<_>` in the attribute, such as
  `Input::<_>`, `Select::<_>::searchable(true)`, or
  `Select::<_>::from(SelectArgs::builder().searchable(true).build())`; the
  derive normalizes the base shape path and resolves `_` to the field's
  form-side type
- generated `FormFields` members and `FormComponents` constructors use the
  source field identifier. Derive-generated inventory records
  shape-level `field_suffix = "..."` metadata when available, or a resolved
  shape-name fallback, for prototyping-specific DOM IDs, event handlers, and
  helper names. Shape-level suffixes must be non-empty identifier suffixes
- field-level `#[koruma(...)]` attributes using Koruma's direct validator
  syntax are accepted by `GpuiForm` and copied onto the generated value holder,
  which allows validating form-side override types without deriving `Koruma` on
  the original model
- when skipped fields are present, the generated value holder keeps builder
  support and exposes `into_original(...)` instead of an unconditional reverse
  conversion. It also exposes a typed `PresentField` enum and
  `present_fields()` snapshot method so examples and tooling can format current
  editable state outside the derive output.
- `skip` cannot be combined with component or hidden intent on the same field

## Component Shape Macros

Use `component_shape_gpui::component_shape!` or
`#[derive(component_shape_gpui::GpuiComponentShape)]` for GPUI shape
declarations. Pair those declarations with
`gpui_form_runtime::shape::GpuiFormComponentShapePolicy` when the shape is used
as a `GpuiForm` field so the form derive can choose required or direct value
storage.

## Feature Flags

- `inventory`: enables `GpuiFormShape` registration for `#[derive(GpuiForm)]`

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the main facade
- [`gpui-form-collection-derive`](../gpui-form-collection-derive/README.md) for
  `#[derive(SelectItem)]` when using derives directly
- [`gpui-form-component`](../gpui-form-component/README.md) for runtime helpers,
  built-in component shape implementations, and `#[derive(InfiniteSelect)]`
  with its `derive` feature
- [`gpui-form-runtime`](../gpui-form-runtime/README.md) for component shape
  contracts and value-binding helpers
- [`gpui-form-schema`](../gpui-form-schema/README.md) when you need metadata
  rather than derives
