# Field intents and generated types

Field intents decide which values render, which values stay in the generated
holder, and which values the application must supply during reconstruction.
Every non-empty-form field must choose exactly one intent.

| Intent | Generated component | Stored in holder | Reconstruction |
|---|---:|---:|---|
| `component(...)` | Yes | Yes | Uses the holder value |
| `hidden` | No | Yes | Uses the holder value |
| `skip` | No | No | Caller supplies the source value |

## Component fields

Use `component(...)` for fields edited by a GPUI widget:

```rust,ignore
#[gpui_form(component(gpui_form_collection::input::Input::<_>))]
name: String,
```

The shape owns construction, rendering, value binding, and the storage policy
used by the generated holder. A configured shape expression can customize
construction:

```rust,ignore
#[gpui_form(
    component(gpui_form_collection::select::Select::<_>.searchable(true))
)]
country: Country,
```

Put `default = ...` inside `component(...)` when the holder should start with a
field-specific value. A component shape also defines how a non-optional field
stores an empty value; see [Component shapes](component_shapes.md#storage-policy).

## Hidden fields

Use `hidden` for values that participate in the holder and conversion without a
component:

```rust,ignore
#[gpui_form(hidden(default = request_context()))]
context: RequestContext,
```

Hidden fields are useful for non-visual values that may still come from a
structured client or application default. They do not create a
`FormFields` member or a `FormComponents` constructor.

## Skipped fields

Use `skip` for source fields the form does not own:

```rust,ignore
#[gpui_form(skip)]
created_by: UserId,
```

Reconstruct a skipped-field model by passing those values to
`holder.into_original(created_by, ...)`. If another field can fail conversion,
the same method returns `Result<Model, Error>`. `holder.present_fields()`
provides a typed snapshot of the editable fields for preview or diagnostic UI.

Do not combine `skip` with `component` or `hidden` on the same field.

## Form-side value types

Use `value(...)` when a widget edits a type different from the source field:

```rust,ignore
#[gpui_form(component(
    gpui_form_collection::input::Input::<_>,
    value(
        type = String,
        from_source = format_account_id,
        try_into_source = parse_account_id,
    )
))]
account_id: AccountId,
```

Write the base form-side type in `type = ...`; source optionality determines
whether the holder stores an optional value. Both `from_source` and one reverse
conversion are required. Use `into_source` for an infallible reverse conversion
or `try_into_source` for a function returning `Result<Source, Error>` where the
error implements `Debug`.

For a struct-level `#[koruma(newtype)]`, `value(koruma_newtype)` edits the inner
value and reconstructs the validated wrapper through Koruma's public newtype
traits.

## Generated types

For `UserProfile`, the generated types have distinct jobs:

| Type | Use |
|---|---|
| `UserProfileFormField` | Identify component-backed fields; each variant's `name()` returns the exact source field name. |
| `UserProfileFormFields` | Store the GPUI entities owned by the form view. |
| `UserProfileFormComponents` | Construct component state with methods named after source fields. |
| `UserProfileFormValueHolder` | Store editable values and defaults, convert back to the model, and expose validation when Koruma integration is enabled. |

Hidden and skipped fields do not produce `UserProfileFormField` variants.
