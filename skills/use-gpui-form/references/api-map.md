# gpui-form API map

Use this reference for application code that consumes `gpui-form`.

## Contents

- [Dependencies](#dependencies)
- [Field and struct syntax](#field-and-struct-syntax)
- [Generated state](#generated-state)
- [Shape selection](#shape-selection)
- [Validation and conversion](#validation-and-conversion)
- [Inventory prototyping](#inventory-prototyping)
- [MCP forms](#mcp-forms)
- [Custom shape routing](#custom-shape-routing)

## Dependencies

Keep `gpui` and `gpui-component` aligned with the repository guidance:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "b077f41a9f26ae5ed7fadfea55a501d34afb25de" }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
gpui-form = "0.6"
gpui-form-collection = "0.6"
gpui-form-collection-derive = "0.6"
gpui-form-component = { version = "0.6", features = ["component-shape", "derive"] }
```

Select only the optional crates the form uses.

| Need | `gpui-form` configuration |
|---|---|
| Normal GPUI form | Default features |
| Inventory metadata | `features = ["inventory"]` |
| MCP in a GPUI app | `features = ["mcp"]` |
| Headless MCP form | `default-features = false, features = ["derive", "mcp"]` |
| MCP Chrono or decimal schemas | Add `chrono` or `rust_decimal` |

## Field and struct syntax

Every non-empty form field has exactly one intent:

```rust
#[gpui_form(component(my::Shape))]
#[gpui_form(component(my::Shape, default = <expr>))]
#[gpui_form(hidden)]
#[gpui_form(hidden(default = <expr>))]
#[gpui_form(skip)]
```

Defaults and conversions are intent-scoped:

```rust
#[gpui_form(component(
    my::Shape,
    value(
        type = FormValue,
        from_source = to_form,
        try_into_source = to_model,
    ),
    default = source_default()
))]
```

Use `into_source` instead of `try_into_source` when reverse conversion is
infallible. Use `value(koruma_newtype)` for the inner value of a struct-level
`#[koruma(newtype)]`. The `type` option names the base form-side type; source
optionality determines optional holder storage.

Field metadata for MCP uses separate `label`, `description`, and repeatable
`example` attributes. Descriptions fall back to field rustdoc.

Common struct attributes are:

```rust
#[gpui_form(empty)]
#[gpui_form(koruma)]
#[gpui_form(koruma(fluent))]
#[gpui_form(no_inventory)]
#[gpui_form(mcp)]
```

`no_inventory` is for generic forms or deliberate inventory exclusion. It is
incompatible with MCP form registration.

## Generated state

For `UserProfile`, the derive emits:

| Type | Role |
|---|---|
| `UserProfileFormField` | Typed identity for component-backed source fields |
| `UserProfileFormFields` | GPUI component entities |
| `UserProfileFormComponents` | Entity-state constructors |
| `UserProfileFormValueHolder` | Editable values, defaults, validation, and conversion |

A hand-wired view:

1. Constructs a holder.
2. Creates each component entity with `*FormComponents`.
3. Stores the entities in `*FormFields`.
4. Retains subscriptions on the owning GPUI entity.
5. Seeds state with `seed_value_binding_state`.
6. Applies events with `value_change`.
7. Renders the shape's declared render component.
8. Validates and converts before submission.

Generated field and constructor names use the source field identifier.
Prototyping-only helper names use the shape's field suffix.

## Shape selection

| Value or behavior | Shape |
|---|---|
| `FromStr + ToString` text-like value | `gpui_form_collection::input::Input::<_>` |
| App-defined text parsing/formatting | `input::ParsedInput::<_, Config>` |
| One enum-like choice | `select::Select::<_>` |
| `Vec<Item>` choices | `combobox::Combobox::<Item>` |
| `bool` | `checkbox::Checkbox` or `switch::Switch` |
| Numeric text with step controls | `number_input::NumberInput::<_>` |
| `f32` or `SliderValue` | `slider::Slider` |
| `gpui::Hsla` | `color_picker::ColorPicker` |
| `chrono::NaiveDate` or date pair | Collection `DatePicker` or `DateRangePicker` |
| OTP value | `otp_input::OtpInput::<_>` |
| Localized date or date pair | Component `DatePicker` or `DateRangePicker` |
| `Vec<PathBuf>` | `gpui_form_component::file_picker::FilePicker` |
| Cascading enum tree | `gpui_form_component::infinite_select::InfiniteSelect::<_>` |

Select values normally derive `SelectItem` and `EnumIter`. Cascading enum trees
derive `InfiniteSelect` and implement `Clone + Default + PartialEq + 'static`;
nested payloads also implement `Default`.

Configure one field through a shape builder:

```rust
#[gpui_form(component(
    gpui_form_collection::select::Select::<_>.searchable(true)
))]
```

Use `Shape.from(options)` for a completed options value. Use
`use-gpui-form-component-shapes` for an app-owned shape or construction
behavior outside a built-in builder.

## Validation and conversion

Add `#[gpui_form(koruma)]` and place Koruma validators on source fields. The
derive copies them to the generated holder, so call `holder.validate()` before
conversion. Use `koruma(fluent)` when the application installs the matching
Fluent validation resources.

Choose conversion from the form contract:

| Contract | Conversion |
|---|---|
| Statically infallible, no skipped fields | `holder.into_original()` |
| Missing required values or fallible reverse conversion | `holder.try_into_original()` |
| Skipped source fields | `holder.into_original(skipped_value, ...)` |

Use `holder.present_fields()` for app-owned debug or preview formatting when
skipped fields prevent automatic reconstruction.

## Inventory prototyping

Enable `inventory` and link the crate that owns the concrete forms into the
generator. Iterate:

```rust,ignore
use gpui_form::schema::registry::{GpuiFormShape, inventory};

for shape in inventory::iter::<GpuiFormShape>() {
    gpui_form_prototyping_core::FormShapeAdapter::new(shape)
        .generate_file(&layout)?;
}
```

Implement `FormLayout` for the target file. Use `parts()` for custom assembly
or path remapping when output belongs to another crate. Format written Rust
with `rustfmt`. Missing render, value-binding, path, or storage metadata is a
shape contract error; fix the shape rather than emitting placeholder UI.

Generic form structs use `#[gpui_form(no_inventory)]`.

## MCP forms

Enable `mcp`, derive Serde support for exposed values, and opt in a concrete
form:

```rust
#[derive(Clone, Debug, serde::Deserialize, gpui_form::GpuiForm, serde::Serialize)]
#[gpui_form(mcp(name = "submit_contact"))]
pub struct ContactRequest {
    #[gpui_form(hidden)]
    pub email: String,
}

#[gpui_form::mcp_submit]
async fn submit_contact(
    request: ContactRequest,
) -> Result<ContactResponse, String> {
    // Application-owned submission.
}
```

The handler is a free synchronous or asynchronous function with one owned
parameter. Use the source model for normal submission or the generated holder
when skipped fields require application-owned context. Responses implement
`Serialize + McpJsonSchema`; errors implement `Display`.

For shared state, add `context(MyContext)` and either implement
`McpContextSubmit<MyContext>` or provide `submit(path)`. Add
`response(MyResponse)` for a precise schema, `map_response(path)` for fallible
response mapping, or `submitter(TraitPath)` when an application trait owns the
submission contract. Register matching forms with
`register_context_submitters(...)`.

Registration choices:

| Need | API |
|---|---|
| Default stdio server | `serve_stdio_blocking()` |
| Existing MCP server | `register(&mut server)` |
| Submit/editor/metadata profile | `register_with_options(...)` |
| Shared MCP tool registry | `tool_registry()` or `tool_registry_with_options(...)` |
| Shared context | `register_context_submitters(...)` |
| Generated prompts | `register_prompt_templates(...)` |
| One manual form | `form::<Form>(&mut server)` |

Editor sessions use `*_edit_open`, `*_edit_patch`, `*_edit_validate`,
`*_edit_submit`, and `*_edit_close`. Retain revisions and send
`expected_revision` for optimistic concurrency. Configure limits and idle
expiry with `McpFormEditorOptions`. Sessions do not mutate live GPUI entities.
MCP servers retain those sessions across calls; tool completion does not
request shutdown.

Registered forms publish descriptor, schema, and examples resources below
`gpui-form://forms/{tool_name}/...`. Field schemas use `McpToolValue`; custom
Serde values normally derive `McpJsonSchema`. Component shapes can attach
value-specific MCP input metadata. Koruma forms publish validation rules and
structured issues.

## Custom shape routing

Switch to `use-gpui-form-component-shapes` when the task:

- defines an application-owned rendered component as a form shape
- wraps state or a component from another crate
- implements shape-level value compatibility or event binding
- selects direct versus required holder storage
- publishes shape-specific MCP input or prototyping suffix metadata

That skill routes generic declaration and rendering details to
`use-component-shape` and `use-component-shape-gpui`.
