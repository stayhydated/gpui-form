# gpui-form-component

GPUI-facing runtime helpers for the `gpui-form` ecosystem.

Applications that use these runtime helpers should depend on this crate
directly. Component-shape contracts live in
[`gpui-form-runtime`](../gpui-form-runtime/README.md) and are re-exported by
`gpui-form` for generated `GpuiForm` field code.

## What It Provides

- `infinite_select`: runtime traits and helpers for cascading enum selects
- `date_picker`: localized runtime state and element wrappers for calendar
  date input
- `file_picker`: native GPUI path selection rendered with `gpui-component`
  controls, plus a derive-backed `FilePicker` form shape when the
  `component-shape` feature is enabled

## Feature Flags

- `derive`: re-exports `#[derive(InfiniteSelect)]`
- `component-shape`: enables built-in `ComponentShape` impls and value-binding
  metadata for `InfiniteSelect`, `DatePicker`, `DateRangePicker`, and
  `FilePicker`

## Infinite Select

Most applications derive `gpui_form_component::InfiniteSelect` by enabling this
crate's `derive` feature and use runtime types from
`gpui_form_component::infinite_select`. This crate owns the runtime trait and
state helpers those derives target.

If you want to depend on the derive crate directly, pair this crate with
[`gpui-form-component-derive`](../gpui-form-component-derive/README.md). The
proc macro resolves `gpui-form-component` as the runtime crate.

```rs
use gpui_form_component::InfiniteSelect;
use gpui_form_component::infinite_select::{InfiniteSelectPath, build_from_path};

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
pub enum Country {
    #[default]
    USA(USAState),
    Canada(CanadaProvince),
    UK,
}
```

Useful runtime types:

- `InfiniteSelectValue`
- `InfiniteSelectItem<T>`
- `InfiniteSelectPath`
- `InfiniteSelectKeyPath`
- `InfiniteSelectKeyPathParseError`
- `InfiniteSelectPathError`
- `InfiniteSelectState<T>`
- `InfiniteSelect<T>`
- `InfiniteSelectEvent<T>`
- `InfiniteSelectLevel<D>`
- `InfiniteSelectSnapshot<T, D>`
- `InfiniteSelectOptions`
- `to_select_items::<T>()`
- `path_from_value(&value)`
- `key_path_from_value(&value)`
- `build_from_path`
- `build_from_key_path`

Manual forms can subscribe to one runtime entity instead of rebuilding nested
child selects themselves:

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

Rendering code can iterate render-ready form fields directly:

```rs
for field in location.read(cx).form_fields() {
    let _ = field;
}
```

For `#[derive(GpuiForm)]`, use the infinite-select component directly:

```rs
#[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>))]
pub location: Country,
```

Enable this crate's `component-shape` feature when using
`InfiniteSelect::<_>` directly as a form shape.

If a form needs non-default infinite-select options such as search or a depth
limit, declare a small `component_shape!` wrapper whose `new` function calls
`InfiniteSelectState::new_with_options(...)`.

Derived `InfiniteSelect` enums expose:

- `PartialEq` alignment with the backing `gpui-component` select value
  comparison
- `variant_label()` for user-facing option titles
- `#[fluent_kv(keys = ["label", "description"], keys_this)]` to emit
  `es-fluent` metadata for application-owned localizers; runtime labels use
  plain fallback names because the runtime trait contract is localizer-free
- `variant_key()` plus `selection_key_path()` for order-independent paths
- `#[tuple_enum(key = "...")]` when persisted keys should not mirror variant names
- `set_child_by_key(...)` / `set_child_by_key_path(...)` for programmatic updates
- `InfiniteSelectKeyPath` implements `Display`, `FromStr`, and serde string serialization
- `set_selected_index_at_depth(...)` / `set_selected_key_at_depth(...)` for incremental updates
- `build_from_path(...)`, `build_from_key_path(...)`, `set_path(...)`, and
  `set_key_path(...)` return `InfiniteSelectPathError` instead of failing
  silently

## Date Picker

This crate provides the localized runtime date-picker that component shapes can wrap.
Its default empty placeholder is plain English fallback copy. Pass
`DatePicker::placeholder(...)` with text rendered through your application-owned
`es-fluent` localizer when a form needs localized or custom copy.

```rs
use gpui_form_component::date_picker::{
    DateDisplayStyle,
    DatePicker,
    DatePickerEvent,
    DatePickerState,
    DateRangePicker,
    DateRangePickerEvent,
    DateRangePickerState,
};
```

Component shapes can store `Entity<DatePickerState>`, render `DatePicker`, and
convert emitted `DatePickerEvent::Change` values with `parse_form_date`.
The selected-date label and embedded calendar popover share the same display
locale: ICU4X formats month names, weekday headers, day/year labels, and the
locale-specific first day of the week.
Manual forms can use `DateRangePickerState`, `DateRangePicker`, and
`DateRangePickerEvent` when they need range selection over the same localized
calendar popover. The ready-made collection date shapes use
`gpui_component::date_picker`; use this runtime directly when the localized
`gpui-form-component` date picker is specifically needed.

For `#[derive(GpuiForm)]`, enable this crate's `component-shape` feature and
use the rendered component type as the form shape:

```rs
#[gpui_form(component(gpui_form_component::date_picker::DatePicker))]
pub birth_date: chrono::NaiveDate;

#[gpui_form(component(gpui_form_component::date_picker::DateRangePicker))]
pub holiday_range: (chrono::NaiveDate, chrono::NaiveDate);
```

## File Picker

This crate provides a native path picker backed by the pinned GPUI git API,
not a separate dialog crate.
Component shapes can use the same runtime.
The default placeholders, native-dialog prompts, browse label, dropped-dialog
error, and selected-count text have plain English fallback copy. Explicit
builder values such as `placeholder(...)`, `prompt(...)`, and
`browse_label(...)` remain caller-provided text; render localized strings through
your application-owned `es-fluent` localizer before passing them in.

```rs
use gpui_form_component::file_picker::{
    FilePicker,
    FilePickerEvent,
    FilePickerState,
};

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

Use `FilePicker::directories()` or `FilePicker::files_or_directories()` when
the dialog should select directories instead of files. Multiple selection is
available through `FilePicker::multiple(true)`.

For `#[derive(GpuiForm)]`, enable this crate's `component-shape` feature and
use the element type as the form shape:

```rs
#[gpui_form(component(gpui_form_component::file_picker::FilePicker))]
pub uploaded_files: Vec<std::path::PathBuf>;
```

## Component Stories

This crate is library-only. The interactive infinite-select, date-picker, and
file-picker storybook gallery lives in
[`gpui-form-component-story`](../gpui-form-component-story/README.md), which
owns the demo UI, launcher binary, and any story-only demo metadata.

Launch the component gallery with:

```sh
cargo run -p gpui-form-component-story
```

## Component Shapes

`ComponentShape` is the runtime contract used by `#[gpui_form(component(Shape))]`.
Generated `GpuiForm` field code also requires `DeclaredComponentShape`, emitted
by `gpui_form_derive::component_shape!` and
`#[derive(gpui_form_derive::ComponentShape)]`. Field shapes should be declared
with one of those macros. Generated field code reaches runtime contracts
through `gpui_form::runtime::shape`; crates that define shapes directly may use
`gpui_form_runtime::shape`. This crate's `component-shape` feature derives that
contract for its built-in rendered component types.

For common external widgets, prefer the reusable shapes in
[`gpui-form-collection`](../gpui-form-collection/README.md). Define your own
shape when the application owns the component or when a collection shape is not
specific enough.

You can declare a reusable wrapper shape with the `gpui-form-derive`
function-like macro. Prefer this form for reusable or generic shapes; it
supports generics, where clauses, outer attributes, and order-independent
metadata.

```rs
gpui_form_derive::component_shape! {
    pub struct EmailInputShape {
        type State = gpui_component::input::InputState;
        component = gpui_component::input::Input;
        value = String;
        field_suffix = "input";
    }
}
```

Or derive the same contract directly on an owned rendered component:

```rs
use crate::state::{TagsState, build};

#[derive(gpui_form_derive::ComponentShape)]
#[gpui_form_shape(
    state = TagsState,
    new = build,
    value = Vec<String>,
    value_storage = direct,
    field_suffix = "input"
)]
pub struct TagsInput {
    state: gpui::Entity<TagsState>,
}
```

`state` and `new` use normal Rust path resolution, so short in-scope names are
fine. `new` accepts a constructor expression in shape declarations. Function
paths and closures are called with
`(window, cx)`; full constructor expressions such as
`TagsState::with_label(window, cx, "tags")` are emitted as written. For
`gpui_form_derive::component_shape!`, omitting `new` calls
`<State>::new(window, cx)`. For `#[derive(ComponentShape)]`, `state = ...` is
required and omitting `new` also calls `<State>::new(window, cx)`.
The `component = ...` option publishes a `RenderComponent` adapter for
generated/prototyping render code, so prefer a type that remains valid from the
consumer crate, such as `gpui_form_collection::checkbox::CheckboxField`.
The value must be a path-like type. `field_suffix = "..."` must be a
non-empty identifier suffix.
Use `value = ...` or `values(...)` to publish the form-side value types a
shape supports; omit those metadata entries only when you provide manual
`ComponentShapeFor<Value>` impls. Do not mix value metadata with manual
compatibility impls in the same `component_shape!` block.
The function-like macro uses semicolons between options.

For component-derived shapes that should participate in generated prototyping
subscriptions, add `value_binding` to `#[gpui_form_shape(...)]` and implement
`ComponentStateValueBinding<T>` on the backing state. Component-derived shapes
delegate their shape-level `ComponentValueBinding<T>` impl through that
state-level contract.
For wrapper shapes declared with `gpui_form_derive::component_shape!`, put the
`ComponentValueBinding<T>` impl inside the macro block to emit the impl with
the shape, and add `value_binding;` to publish shape-level value-binding
metadata for generated prototyping subscriptions.

Reusable shapes can also publish `ComponentPrototyping` metadata through the
shape contract.
Set `field_suffix = "..."` through `gpui_form_derive::component_shape!`, or
`#[gpui_form_shape(...)]` so inventory and prototyping generators can emit
helper names such as `on_email_input_event` without deriving that suffix from
the shape type. Generated form entity fields and constructors use the source
field name, such as `email`.
Generated value-binding scaffolds can use `ComponentStateOf`, `ComponentEventOf`,
`seed_value_binding_state`, and `form_value_change` to avoid repeating
associated-type projections at every call site.
Direct `ComponentPrototyping::field_suffix(...)` calls validate the same
non-empty ASCII identifier suffix contract in const evaluation.

Set `value_storage = direct` on a reusable shape when the component can
synthesize a missing value. Generated forms then inherit direct `T` holder
storage from the shape; consuming field attributes do not accept a
required-value override.

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the `GpuiForm` facade
- [`gpui-form-component-derive`](../gpui-form-component-derive/README.md) when
  you want only the `InfiniteSelect` derive plus this runtime layer
- [`gpui-form-runtime`](../gpui-form-runtime/README.md) for component shape
  contracts and value-binding helpers
- [`gpui-component`](https://github.com/longbridge/gpui-component) for the
  upstream date-picker widget and other base components
- [`gpui-form-schema`](../gpui-form-schema/README.md) for metadata and inventory
- [`gpui-form-prototyping-core`](../gpui-form-prototyping-core/README.md) for
  scaffold generation
