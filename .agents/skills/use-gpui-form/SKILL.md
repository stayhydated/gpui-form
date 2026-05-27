---
name: use-gpui-form
description: "Use when Codex needs to build user-facing application forms with gpui-form, including adding #[derive(GpuiForm)] to app structs, choosing gpui_form component attributes, using generated form fields/components/value holders, wiring SelectItem or InfiniteSelect enums, and adding component shapes."
---

# Use GPUI Form

## Scope Boundary

Treat this skill as a hosted public-usage guide for `gpui-form` consumers. Use
it only for user-facing application workflows: deriving forms on app models,
choosing component attributes, using generated form state, wiring select and
infinite-select enums, and adding app-owned component shapes.

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

4. Add `GpuiForm` to a normal Rust struct and annotate each visible field with
   a component.
5. Use generated types named from the source struct, such as
   `UserProfileFormFields`, `UserProfileFormComponents`, and
   `UserProfileFormValueHolder`.
6. Use `#[gpui_form(default = ...)]` for initial form values,
   `#[gpui_form(skip)]` for model fields that should not render as widgets, and
   `#[gpui_form(type = ..., from = ..., into = ..., component = ...)]` when the
   UI edits a form-side type that differs from the model field. Text input
   prototyping parses non-`String` form-side types with `FromStr`.
7. Use paths such as `gpui_form_component::date_picker`,
   `gpui_form_component::file_picker`, and
   `gpui_form_component::infinite_select` for helper state.
   Directly-reusable collection shapes are also available as
   `gpui_form_collection::date_picker::DatePicker`,
   `gpui_form_collection::date_picker::DateRangePicker`,
   `gpui_form_collection::combobox::Combobox::<_>`,
   `gpui_form_collection::number_input::NumberInput::<_>`,
   `gpui_form_collection::file_picker::FilePicker`,
   `gpui_form_collection::slider::Slider`,
   `gpui_form_collection::color_picker::ColorPicker`, and
   `gpui_form_collection::otp_input::OtpInput::<_>`.

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
    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
    pub username: Option<String>,

    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
    pub age: Option<u32>,

    #[gpui_form(
        component = gpui_form_collection::select::Select::<_>.requires_value(false),
        default = Country::France
    )]
    pub country: Country,
}
```

Common patterns:

- For selects, derive `SelectItem` from `gpui-form-collection-derive` on enum-like values and `EnumIter` when the app needs iteration-backed choices.
- For cascading or nested selects, derive `InfiniteSelect` from `gpui-form-component` with its `derive` feature and `PartialEq` on the enum tree, then use `component = gpui_form_component::infinite_select::InfiniteSelect::<_>` or a dedicated `ComponentShape` wrapper for custom search/depth options.
- Non-optional component fields default to required holder storage; add `.requires_value(false)` when a component can safely synthesize a missing value.
- For app-owned widgets, derive `ComponentShape` from `gpui-form-derive` on a state type or declare a reusable wrapper shape with `gpui_form_derive::component_shape!`.
- For value-bound component shapes, implement `gpui_form_component::shape::ComponentValueBinding<T>` on the shape and use `.value_binding()` in the `component = Shape` expression when the shape does not already publish `VALUE_BINDING`; the trait's associated `Event` is the actual emitted event enum, external wrappers map upstream events to `FormValueChange<T>`, and owned states can mark the shape with `OwnedComponentValueBinding<T>`.
- Let reusable component shapes publish prototyping names with `field_suffix = "..."` when they will feed prototyping output; collection components already publish suffixes such as `input`, `select`, `combobox`, `checkbox`, `switch`, `number_input`, `slider`, `color_picker`, `date_picker`, `date_range_picker`, `file_picker`, and
  `otp_input`, and shapes without metadata fall back to the shape-name heuristic.
- Format written inventory-prototyping scaffolds with `rustfmt`; the workspace `examples/prototyping` generator does this before reporting completion.
- Keep consumer code focused on app models, form state, rendering, and app-owned components.
