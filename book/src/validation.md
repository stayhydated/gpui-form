# Validation and conversion

Validate the generated holder, then use the conversion method emitted for the
form's field contracts. Validation reports invalid editable values; conversion
reconstructs the source model and can separately report a missing required
value or a failed reverse conversion.

| Form contract | Generated conversion |
|---|---|
| No skipped fields; reconstruction is statically infallible | `holder.into_original()` returns the model. |
| No skipped fields; the derive emits a checked path | `holder.try_into_original()` returns `Result<Model, Error>`. This includes fallible conversions and shape-policy component fields without a declared field default. |
| One or more skipped fields | `holder.into_original(skipped_value, ...)` accepts those values and returns either the model or `Result<Model, Error>`, depending on the remaining fields. |

A non-optional shape-backed field can still start empty when its shape uses
required storage and no `default = ...` is declared. Fallible holder conversion
reports that missing value. When the form enables Koruma integration, generated
`validate()` reports it too.

The derive emits the checked conversion method for any non-defaulted
shape-policy component because the policy is resolved through the shape type.
A direct-storage policy always supplies a value, so that checked conversion
cannot fail at runtime unless another field has a fallible conversion.

## Validate with Koruma

Koruma validation can run directly against form-side values:

```rust,ignore
use gpui_form::GpuiForm;
use koruma::Koruma;
use koruma_collection::{collection::NonEmptyValidation, numeric::RangeValidation};

#[derive(Clone, Debug, GpuiForm)]
#[gpui_form(koruma)]
struct Registration {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    #[koruma(NonEmptyValidation::<_>)]
    username: String,

    #[gpui_form(component(gpui_form_collection::number_input::NumberInput::<_>))]
    #[koruma(RangeValidation::<_>.min(18).max(120))]
    age: u8,
}
```

`GpuiForm` copies the field validators to `RegistrationFormValueHolder` and
derives its Koruma implementation. The `Koruma` import brings the `validate()`
method into scope; the source model only needs its own `Koruma` derive when the
application also validates that model directly. All configured validators run
in attribute order:

```rust,ignore
let holder = RegistrationFormValueHolder::default();

if let Err(errors) = holder.validate() {
    if errors.username().non_empty_validation().is_some() {
        eprintln!("username must not be empty");
    }
}
```

Use `#[gpui_form(koruma(fluent))]` when the application has enabled the Koruma
Fluent features and installed its application-owned localizer. The generated
holder derives `KorumaAllFluent` and exposes localized failed-validator values
through `all()`; the source model does not need that derive unless it is also
validated directly.

## Convert validated newtypes

For `#[koruma(newtype)]` source fields, use `value(koruma_newtype)` inside the
field intent. The generated holder edits the inner value and reconstructs the
validated wrapper through Koruma's public newtype traits.

Call the fallible conversion method after `validate()`. Validation is still
useful even when conversion is checked because it can expose field-specific
validator errors instead of only the first reconstruction failure.

## Troubleshooting

| Symptom | Cause and action |
|---|---|
| `validate()` reports a required field with no Koruma attribute | The component shape uses required storage. Supply a value or add an intent-scoped `default = ...`. |
| `try_into_original()` fails after Koruma validation succeeds | A reverse conversion failed or a required value was still absent. Inspect the conversion error's field name. |
| Fluent validator types do not compile or messages cannot be resolved | Enable `koruma/fluent` and the matching `koruma-collection` Fluent feature, then initialize the application's Fluent resources. |
