[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
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
| `branch = "master"` | `branch = "main"` | `rev = "832c17e8192e2e1d472f0751e7cef2af84ded622"` |

## Installation

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "832c17e8192e2e1d472f0751e7cef2af84ded622" }
gpui-component = { git = "https://github.com/longbridge/gpui-component", branch = "main" }

gpui-form = "*"

# Optional: runtime component helpers, built-in component shapes,
# and the InfiniteSelect derive
gpui-form-component = { version = "*", features = ["component-shape", "derive"] }

# Optional: ready-made component shapes and select derive support
gpui-form-collection = "*"
gpui-form-collection-derive = "*"

# Optional: ComponentShape derive and component_shape! shape macro
gpui-form-derive = "*"

# Optional: inventory registration for prototyping/code generation
# gpui-form = { version = "*", features = ["inventory"] }
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
    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    pub username: Option<String>,

    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    pub age: Option<u32>,

    #[gpui_form(
        gpui_form_collection::select::Select::<_>,
        default = Country::France
    )]
    pub country: Country,

    #[gpui_form(gpui_form_collection::checkbox::Checkbox)]
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
the shape expression directly in the attribute:

- `#[gpui_form(my::Shape)]`
- `#[gpui_form(my::Shape.component(my::ui::Widget))]`
- `#[gpui_form(my::Shape.field_suffix("input"))]`
- `#[gpui_form(gpui_form_collection::input::Input::<_>)]`
- `#[gpui_form(gpui_form_collection::select::Select::<_>)]`
- `#[gpui_form(gpui_form_collection::combobox::Combobox::<Country>)]`
- `#[gpui_form(gpui_form_collection::checkbox::Checkbox)]`
- `#[gpui_form(gpui_form_collection::switch::Switch)]`
- `#[gpui_form(gpui_form_collection::number_input::NumberInput::<_>)]`
- `#[gpui_form(gpui_form_collection::slider::Slider)]`
- `#[gpui_form(gpui_form_collection::color_picker::ColorPicker)]`
- `#[gpui_form(gpui_form_collection::date_picker::DatePicker)]`
- `#[gpui_form(gpui_form_collection::date_picker::DateRangePicker)]`
- `#[gpui_form(gpui_form_collection::otp_input::OtpInput::<_>)]`
- `#[gpui_form(gpui_form_component::date_picker::DatePicker)]`
- `#[gpui_form(gpui_form_component::date_picker::DateRangePicker)]`
- `#[gpui_form(gpui_form_component::file_picker::FilePicker)]`
- `#[gpui_form(gpui_form_component::infinite_select::InfiniteSelect::<_>)]`

The `gpui_form_component` shape entries above require that crate's
`component-shape` feature. Infinite-select field types also need the
`InfiniteSelect` derive, available through the same crate's `derive` feature or
by depending on `gpui-form-component-derive` directly.

The shape expression is parsed as attribute metadata; generated runtime
construction delegates to `ComponentShape::new`. `gpui-form` treats every
component as a custom shape contract and does not inspect the shape path for
built-in component categories.
Single-segment local shape identifiers should be UpperCamel, such as
`EmailInputShape`; lowercase bare words are reserved for field options, and the
supported bare options are `hidden` and `skip`. Use a qualified path such as
`my::Shape` when the shape is not a local single-segment type.

Component shapes own the default required-value policy for non-optional fields.
Shapes that can safely synthesize a missing value, such as the built-in inputs,
selects, toggles, sliders, OTP inputs, file picker, and infinite select, keep
generated value-holder storage as `T`. Shapes that require a present value use
`Option<T>` storage; generated `validate()` reports a missing value and
conversion back to the source model fails when the value is missing. Those
fallible holder-to-model paths expose `holder.try_into_original()`; infallible
paths expose `holder.into_original()` and implement `From` when the derive can
prove infallibility directly. Define this behavior on the reusable component
shape with `value_storage = direct`. Required shape-backed fields without a
declared field default keep only `try_into_original()` so missing values cannot
be hidden behind a panic-capable infallible API.

Common field-level helpers:

- Every field must choose exactly one intent: a component shape expression,
  `hidden`, or `skip`.
- `#[gpui_form(hidden)]` keeps a field in the generated value holder without
  generating a GPUI component.
- `#[gpui_form(hidden, default = <expr>)]` seeds a hidden generated value holder
  field.
- `#[gpui_form(<shape>, default = <expr>)]` seeds a component-backed generated
  value holder field.
- `#[gpui_form(skip)]` excludes a field from generated form widgets while still
  allowing prefill from the original model. It cannot be combined with
  component, hidden, default, or conversion options on the same field.
- `#[gpui_form(type = <form_type>, from = <expr>, into = <expr>)]` lets the
  generated form edit a type that differs from the original field type; combine
  it with either a component shape or `hidden`.
- `gpui_form_collection::input::Input::<_>` and
  `gpui_form_collection::number_input::NumberInput::<_>` parse non-`String`
  form-side value types with `FromStr` in prototyping output, so value objects
  can use `type`, `from`, and `into` while the source model keeps its storage
  type. `gpui_form_collection::otp_input::OtpInput::<_>` also uses `FromStr`
  for non-`String` form-side value types.
- Generic component expressions use Rust expression turbofish syntax, such as
  `Input::<_>`. The derive normalizes that path and resolves `_` to
  the field's form-side type.
- generated form field/helper suffixes use field-level `.field_suffix(...)`
  when supplied; otherwise they derive a suffix from the explicit component
  type or shape type name, such as `birth_date_date_picker`. Derive-generated
  inventory records that resolved suffix so prototyping output uses the same
  field and handler names. Field-level and shape-level suffixes must be
  non-empty ASCII identifier suffixes; manual
  `ComponentPrototyping::field_suffix(...)` calls validate the same contract in
  const evaluation.
- Field-level `#[koruma(...)]` attributes are accepted by `GpuiForm` and copied
  onto the generated value holder, including fields that use `type`, `from`,
  and `into` to validate a form-side type.

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

Common struct-level helpers:

- `#[gpui_form(empty)]` marks an intentional empty form.
- `#[gpui_form(koruma)]` enables Koruma-backed validation wiring.
- `#[gpui_form(koruma(fluent))]` enables Koruma validation plus fluent error
  rendering.

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
    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub username: String,

    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    #[koruma(RangeValidation::<_>::builder().min(18).max(120))]
    pub age: Option<u32>,
}
```

When validation is enabled:

- required-value semantics are preserved when the generated holder represents
  required fields as `Option<T>`
- builder-chain Koruma attrs are mirrored
- generated value-holder validation uses the same validator set as the source
  struct

## Component Shapes

There are three supported component-shape workflows.

### 1. Use a collection component

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct Account {
    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    pub code: AccountCode,
}
```

The `_` generic is resolved to the field's form-side type, including any
`#[gpui_form(type = ...)]` override. `Combobox<T>` is the exception: its
generic is the selected item type, so a `Vec<Country>` field uses
`Combobox::<Country>`. The collection combobox publishes
`value_storage = direct`: an empty selection is emitted as
`FormValueChange::Clear`, so optional fields clear to `None` and
non-optional `Vec<T>` fields reset to their declared
`#[gpui_form(default = ...)]` when present, otherwise `Vec::default()`.

### 2. Derive on an owned component

```rs
use gpui_form::GpuiForm;
use gpui_form_derive::ComponentShape;

#[derive(ComponentShape)]
#[gpui_form_shape(state = TagsInputState, field_suffix = "input")]
pub struct TagsInput {
    state: gpui::Entity<TagsInputState>,
}

pub struct TagsInputState;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct PostEditor {
    #[gpui_form(TagsInput)]
    pub tags: Option<Vec<String>>,
}
```

`new` accepts a constructor expression. Use a function path or closure when the
macro should pass `(window, cx)` for you, or use a full constructor expression
such as `TagsInputState::with_label(window, cx, "tags")` when the expression
should be emitted as written. If `new` is omitted, the derive calls
`<State>::new(window, cx)`.

### 3. Declare a reusable external shape

```rs
gpui_form_derive::component_shape! {
    pub struct EmailInputShape {
        type State = gpui_component::input::InputState;
        component = gpui_component::input::Input;
        field_suffix = "input";
    }
}

#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct ContactForm {
    #[gpui_form(EmailInputShape)]
    pub email: String,
}
```

`component_shape!` creates a local zero-sized shape type, so downstream crates
can attach the `gpui-form` contract to external component state without running
into Rust's orphan rules.
It uses `new`, `component`, `value_storage = require_value|direct`, `value_binding`, and
`field_suffix` metadata, plus `type State = ...` for the wrapped external
state type. If `new` is omitted, the macro calls `<State>::new(window, cx)`.
`component = ...` must be a path-like type, and `field_suffix = "..."` must be
a non-empty ASCII identifier suffix.
Separate metadata entries with semicolons.
The block may also contain `impl` items. A nested `ComponentValueBinding<T>`
impl is emitted with the shape. Add `value_binding;` when that wrapper should
publish shape-level value-binding metadata for prototyping subscriptions.

Generated `GpuiForm` component fields compile against
`gpui_form::runtime::shape`, which is re-exported by the facade crate. Normal
application crates do not add `gpui-form-runtime` just because a field uses a
component shape. Add `gpui-form-runtime` directly only for lower-level shape
crates that use `#[derive(ComponentShape)]`, `component_shape!`,
`component_value_binding`, or manual runtime trait implementations; those
macros emit direct runtime paths and resolve renamed runtime dependencies.
If you implement `ComponentShape` manually instead of using the derive or
`component_shape!`, also implement `ComponentShapeFor<Value>` for each
form-side value type the shape supports, and set both policy associated types:
`RequiredValuePolicy` controls value-holder storage, and `ValueBindingPolicy`
is usually `NoComponentValueBinding` unless the shape should inherit
`ComponentValueBinding<T>` synchronization by default.

Component-derived shapes can opt into generated value synchronization by adding
`value_binding` to `#[gpui_form_shape(...)]` and placing
`#[gpui_form_derive::component_value_binding]` on the backing state's binding
impl:

```rs
pub struct TagsInputState;

#[gpui_form_derive::component_value_binding]
impl gpui_form_runtime::shape::ComponentValueBinding<Vec<String>> for TagsInputState {
    type Event = TagsInputEvent;
    /* seed_value_binding_state and form_value_change */
}
```

Wrapper shapes can put the binding impl directly inside `component_shape!`:

```rs
gpui_form_derive::component_shape! {
    pub struct EmailInputShape {
        type State = gpui_component::input::InputState;
        component = gpui_component::input::Input;
        value_binding;

        impl gpui_form_runtime::shape::ComponentValueBinding<String> for EmailInputShape {
            type Event = gpui_component::input::InputEvent;
            /* seed_value_binding_state and form_value_change */
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
`FilePicker` directly in `#[gpui_form(...)]`; the shape stores
`FilePickerState` as its backing entity state.

## Date Conversion

`date_picker` fields can edit a different form-side type than the original model
field by combining `type`, `from`, and `into`:

```rs
#[derive(Clone, Debug, gpui_form::GpuiForm)]
pub struct User {
    #[gpui_form(
        gpui_form_collection::date_picker::DatePicker,
        type = chrono::NaiveDate,
        from = to_form_date,
        into = to_model_timestamp
    )]
    pub birth_date: Option<Timestamp>,
}
```

This pattern is useful when the model stores a domain-specific timestamp type
but the UI should edit a calendar date.
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

See [`examples/prototyping`](../../examples/prototyping) for a complete
generator that reads shape inventory, clears stale generated form modules,
writes scaffolded GPUI form files into `examples/some-lib-forms/src/forms`, and
formats them with `rustfmt`. The Storybook form titles generated by that example
use the GPUI app context so they follow the active Storybook locale through
`gpui-es-fluent`. Value-bound shape-backed fields use helper functions and
runtime aliases from `gpui_form::runtime::shape` for state and event
projections, keeping handler signatures based on names such as
`on_email_input_event`. The adapter uses the derive-emitted
`holder_conversion_can_fail` inventory flag so generated debug output matches
the holder's `into_original` or `try_into_original` API.

## Examples

[`examples/README.md`](../../examples/README.md) is the canonical index for runnable
workspace examples.

- `cargo run -p some-lib-forms`: browse generated forms in a storybook-style UI
- `cargo run -p gpui-form-component-story`: browse runtime component demos
- `cargo run -p prototyping`: regenerate the `some-lib-forms` scaffolded GPUI
  form files from shape inventory
