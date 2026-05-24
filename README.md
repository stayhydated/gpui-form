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

## Compatibility

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

# Optional: runtime component contracts/helpers and the InfiniteSelect derive
gpui-form-component = { version = "*", features = ["derive"] }

# Optional: ready-made component shapes and select derive support
gpui-form-collection = "*"
gpui-form-collection-derive = "*"

# Optional: CustomComponent derive and custom_component! shape macro
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
    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
    pub username: Option<String>,

    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
    pub age: Option<u32>,

    #[gpui_form(
        component = gpui_form_collection::select::Select::<_>,
        default = Country::France
    )]
    pub country: Country,

    #[gpui_form(component = gpui_form_collection::checkbox::Checkbox)]
    pub subscribe: bool,
}
```

`#[derive(GpuiForm)]` generates the typed form support around that struct:

- `UserProfileFormFields` for GPUI entity state
- `UserProfileFormComponents` constructors for those fields
- `UserProfileFormValueHolder` for typed editing, defaults, validation, and
  conversion back into the original model

## Component Syntax

`gpui-form` parses contract-backed component expressions:

- `#[gpui_form(component = my::Shape)]`
- `#[gpui_form(component = my::Shape.component(my::ui::Widget))]`
- `#[gpui_form(component = my::Shape.value_binding())]`
- `#[gpui_form(component = my::Shape.field_suffix("input"))]`
- `#[gpui_form(component = gpui_form_collection::input::Input::<_>)]`
- `#[gpui_form(component = gpui_form_collection::select::Select::<_>::searchable(true).partial(true))]`
- `#[gpui_form(component = gpui_form_collection::checkbox::Checkbox)]`
- `#[gpui_form(component = gpui_form_collection::switch::Switch)]`
- `#[gpui_form(component = gpui_form_component::infinite_select::InfiniteSelect::<_>::searchable(true).max_depth(3))]`

The expression is parsed as attribute metadata; generated runtime construction
delegates to `CustomComponentShape::new`. Select and infinite-select behavior
uses Koruma-style direct setter chains that expand to bon builders inside the
derive macro.
Generated required-value behavior is inferred internally from known components:
`gpui_form_collection::input::Input` allows required source fields to be absent
in the holder until validation or conversion, while
`gpui_form_collection::select::Select`, `gpui_form_collection::checkbox::Checkbox`,
`gpui_form_collection::switch::Switch`, and
`gpui_form_component::infinite_select::InfiniteSelect` keep non-optional source fields
as `T`.

Common field-level helpers:

- `#[gpui_form(default = <expr>)]` seeds the generated value holder.
- `#[gpui_form(skip)]` excludes a field from generated form widgets while still
  allowing prefill from the original model.
- `#[gpui_form(type = <form_type>, from = <expr>, into = <expr>)]` lets the
  generated form edit a type that differs from the original field type.
- `gpui_form_collection::input::Input::<_>` parses non-`String` form-side
  value types with `FromStr` in prototyping output, so value objects can use
  `type`, `from`, and `into` while the source model keeps its storage type.
- Generic component expressions use Rust expression turbofish syntax, such as
  `Input::<_>`. The derive normalizes that path and resolves `_` to
  the field's form-side type.
- custom component prototyping metadata drives generated field/helper suffixes
  when present. Collection components publish suffixes such as `input`,
  `select`, `checkbox`, and `switch`; reusable app shapes can use
  `field_suffix = "..."`, and otherwise the generator falls back to the shape
  name heuristic.
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

Common struct-level helpers:

- `#[gpui_form(empty)]` marks an intentional empty form.
- `#[gpui_form(koruma)]` enables Koruma-backed validation wiring.
- `#[gpui_form(koruma(fluent))]` enables Koruma validation plus fluent error
  rendering.

## Infinite Select Runtime

Infinite-select custom fields are backed by
`gpui_form_component::infinite_select::InfiniteSelect`, which owns the
root and child `SelectState`s, exposes render-ready level snapshots, and emits
a single typed change event with the rebuilt nested value, both path forms, the
previous paths, and the changed depth.

```rs
use gpui_form_component::infinite_select::{InfiniteSelect, InfiniteSelectEvent};

let location = cx.new(|cx| {
    InfiniteSelect::new(Country::default(), window, cx)
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
    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub username: String,

    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
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

## Custom Components

There are three supported custom-component workflows.

### 1. Use a collection component

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct Account {
    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
    pub code: AccountCode,
}
```

The `_` generic is resolved to the field's form-side type, including any
`#[gpui_form(type = ...)]` override.

### 2. Derive directly on an owned state type

```rs
use gpui_form::GpuiForm;
use gpui_form_derive::CustomComponent;

#[derive(Clone, Debug, CustomComponent)]
#[gpui_form_custom(new = Self::new, component = TagsInput, field_suffix = "input")]
pub struct TagsInputState;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct PostEditor {
    #[gpui_form(component = TagsInputState)]
    pub tags: Option<Vec<String>>,
}
```

### 3. Declare a reusable external shape

```rs
gpui_form_derive::custom_component! {
    pub struct EmailInputShape {
        type State = gpui_component::input::InputState;
        new = gpui_component::input::InputState::new;
        component = gpui_component::input::Input;
        field_suffix = "input";
    }
}

#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct ContactForm {
    #[gpui_form(component = EmailInputShape)]
    pub email: String,
}
```

`custom_component!` creates a local zero-sized shape type, so downstream crates
can attach the `gpui-form` contract to external component state without running
into Rust's orphan rules.

Custom components can also opt into generated value synchronization by
implementing `gpui_form_component::custom::CustomComponentValueBinding<T>` on the shape.
For one-off fields, add `value_binding` to the custom component options. For a
reusable component, put that metadata on the shape:

```rs
use gpui_form_derive::CustomComponent;

#[derive(Clone, Debug, CustomComponent)]
#[gpui_form_custom(
    new = Self::new,
    component = TagsInput,
    value_binding,
    field_suffix = "input"
)]
pub struct TagsInputState;
```

External component wrappers, such as `gpui-component` inputs, map their native
event into `FormValueEvent<T>` with `form_value_event`. Components that own
their state can emit `FormValueEvent<T>` directly and mark the shape with
`OwnedCustomComponentValueBinding<T>`. The binding remains application-owned;
`gpui-form` only calls `seed_value_binding_state` and `form_value_event`.

Runtime helpers are available from `gpui_form_component`; `gpui-form` does not
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

## Date Conversion

`date_picker` fields can edit a different form-side type than the original model
field by combining `type`, `from`, and `into`:

```rs
#[derive(Clone, Debug, gpui_form::GpuiForm)]
pub struct User {
    #[gpui_form(
        type = chrono::NaiveDate,
        from = to_form_date,
        into = to_model_timestamp,
        component = crate::DatePickerShape
    )]
    pub birth_date: Option<Timestamp>,
}
```

This pattern is useful when the model stores a domain-specific timestamp type
but the UI should edit a calendar date.
The default empty placeholder is plain English fallback copy; pass
`DatePicker::placeholder(...)` when a form needs localized or custom copy. The
selected-date label and calendar popover use ICU4X for localized month names,
weekday headers, day/year labels, and locale-specific week starts. Manual
runtime code can use `DateRangePicker` and `DateRangePickerState` for range
selection. To use this runtime from `GpuiForm`, define an application or
collection shape that implements
`gpui_form_component::custom::CustomComponentShape`.

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

See [`examples/prototyping`](examples/prototyping) for a complete generator
that reads shape inventory, clears stale generated form modules, writes
scaffolded GPUI form files, and formats them with `rustfmt`. The Storybook form
titles generated by that example use the GPUI app context so they follow the
active Storybook locale. Value-bound custom fields use helper functions and runtime aliases from
`gpui_form_component::custom` for state and event projections, keeping handler
signatures based on names such as `on_email_input_event`.

## Examples

[`examples/README.md`](examples/README.md) is the canonical index for runnable
workspace examples.

- `cargo run -p some-lib-forms`: browse generated forms in a storybook-style UI
- `cargo run -p gpui-form-component-story`: browse runtime component demos
- `cargo run -p prototyping`: generate scaffolded GPUI form files from shape
  inventory
