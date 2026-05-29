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
`ComponentShape` implementations, or shape-level value bindings, use
`use-gpui-form-component-shapes`.

Do not use this skill as a contributor guide for `gpui-form` repository
internals. For build, test, format, lint, maintenance, release, or architecture
work, read the repository source, `AGENTS.md`, and the relevant crate
documentation directly.

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
6. Put the component shape directly in `#[gpui_form(...)]`, use
   `#[gpui_form(hidden)]` for value-holder-only fields,
   `#[gpui_form(default = ...)]` with either a component or `hidden` for initial form values,
   `#[gpui_form(skip)]` for model fields that should not render as widgets, and
   `#[gpui_form(Shape, type = ..., from = ..., into = ...)]` when the
   UI edits a form-side type that differs from the model field. Do not combine
   `skip` with component, hidden, default, type, from, or into options on the same
   field. Text input prototyping parses non-`String` form-side types with
   `FromStr`.
7. Use paths such as `gpui_form_component::date_picker`,
   `gpui_form_component::file_picker`, and
   `gpui_form_component::infinite_select` for helper state.
   Generated `GpuiForm` component fields use the facade path
   `gpui_form::runtime::shape`, so normal application crates do not add
   `gpui-form-runtime` just because a field uses a component shape.
   Use `gpui_form_component::file_picker::FilePicker` as a ready-made form
   shape when `gpui-form-component` has its `component-shape` feature enabled;
   generated fields still store `FilePickerState` as the backing entity state.
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
    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    pub username: Option<String>,

    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    pub age: Option<u32>,

    #[gpui_form(
        gpui_form_collection::select::Select::<_>,
        default = Country::France
    )]
    pub country: Country,
}
```

Common patterns:

- For selects, derive `SelectItem` from `gpui-form-collection-derive` on enum-like values and `EnumIter` when the app needs iteration-backed choices. `SelectItem` uses variant-name fallback labels by default and only needs `Display` with `#[select_item(display)]`.
- For cascading or nested selects, derive `InfiniteSelect` from `gpui-form-component` with its `derive` feature and `PartialEq` on the enum tree. Enable `gpui-form-component`'s `component-shape` feature when using `#[gpui_form(gpui_form_component::infinite_select::InfiniteSelect::<_>)]` directly.
- Component shapes own the default required-value policy for non-optional fields. Use plain built-in value-synthesizing shapes such as `Input::<_>`, `Select::<_>`, `Combobox::<Item>`, `Checkbox`, `Switch`, `NumberInput::<_>`, `Slider`, `OtpInput::<_>`, `FilePicker`, and `InfiniteSelect::<_>`. Date picker and color picker shapes should usually back optional fields or receive a default when the model field is required. Required shape-backed values are visible to generated `validate()` as well as fallible holder-to-model conversion.
- Convert generated holders with `holder.try_into_original()` when conversion can fail, `holder.into_original()` when it is statically infallible, or `holder.into_original(skipped_value, ...)` when the source model has skipped fields that the form cannot edit. Required shape-backed fields without a declared default keep only the fallible holder-to-model path.
- `Combobox::<Item>` treats an empty selection as `FormValueChange::Clear`; optional fields clear to `None`, while non-optional `Vec<Item>` fields reset to their declared `#[gpui_form(default = ...)]` when present, otherwise `Vec::default()`.
- For app-owned widgets, external component/state wrappers, custom search/depth options, reusable `ComponentShape` implementations, or shape-level value bindings, use `use-gpui-form-component-shapes`.
- Collection and component-owned shapes publish prototyping suffixes such as `input`, `select`, `combobox`, `checkbox`, `switch`, `number_input`, `slider`, `color_picker`, `date_picker`, `date_range_picker`,
  `file_picker`, `infinite_select`, and `otp_input`. Generated form identifiers and derive-emitted
  inventory use field-level `.field_suffix(...)` first, then the explicit component type or shape
  type name; field-level suffixes must be non-empty ASCII identifier suffixes. Manual
  `ComponentPrototyping::field_suffix(...)` calls validate the same suffix contract.
- Format written inventory-prototyping scaffolds with `rustfmt`; the workspace `examples/prototyping` generator does this before reporting completion.
- Storybook-style GPUI scaffolds that localize generated labels or messages should call `gpui_es_fluent::localize_label` and `gpui_es_fluent::localize_message` with the active `gpui::App` context.
- Keep consumer code focused on app models, form state, rendering, and app-owned components.
