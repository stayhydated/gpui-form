# gpui-form-component

GPUI-facing runtime helpers for the `gpui-form` ecosystem.

Applications that use these runtime helpers should depend on this crate
directly. `gpui-form` does not re-export the component runtime or its derive
macros.

## What It Provides

- `infinite_select`: runtime traits and helpers for cascading enum selects
- `date_picker`: localized runtime state and element wrappers for calendar
  date input
- `file_picker`: native GPUI path selection rendered with `gpui-component`
  controls, plus a derive-backed `FilePickerState` form shape when the
  `component-shape` feature is enabled
- `shape`: the runtime contract for user-defined component state

## Feature Flags

- `derive`: re-exports `#[derive(InfiniteSelect)]`
- `component-shape`: enables built-in `ComponentShape` impls and value-binding
  metadata for `InfiniteSelect`, `DatePickerState`, `DateRangePickerState`, and
  `FilePickerState`

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
- `InfiniteSelect<T>`
- `InfiniteSelectField<T>`
- `SearchableInfiniteSelect<T>`
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

Rendering code can iterate render-ready form fields directly:

```rs
for field in location.read(cx).form_fields() {
    let _ = field;
}
```

For `#[derive(GpuiForm)]`, use the infinite-select component directly:

```rs
#[gpui_form(gpui_form_component::infinite_select::InfiniteSelect::<_>)]
pub location: Country,
```

Enable this crate's `component-shape` feature when using `InfiniteSelect::<_>`
directly as a form shape.

If a form needs non-default infinite-select options such as search or a depth
limit, define a small `ComponentShape` wrapper whose `new` function calls
`InfiniteSelect::new_with_options(...)`.

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
use the state type as the form shape:

```rs
#[gpui_form(gpui_form_component::file_picker::FilePickerState)]
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

`shape::ComponentShape` is the contract used by
`#[gpui_form(Shape)]` and `#[gpui_form(component = Shape)]`.

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
        field_suffix = "input";
    }
}
```

Or derive the same contract directly on a state type:

```rs
#[derive(gpui_form_derive::ComponentShape)]
#[gpui_form_shape(
    new = crate::state::build,
    component = crate::ui::TagsInput,
    value_binding,
    field_suffix = "input"
)]
pub struct TagsState;
```

In both forms, `new` accepts a constructor expression. Function paths and
closures are called with `(window, cx)`; full constructor expressions such as
`Self::with_label(window, cx, "tags")` are emitted as written. For
`gpui_form_derive::component_shape!`, omitting `new` calls
`<State>::new(window, cx)`.
`value_binding` is a bare flag; omit it when the shape should not publish
shape-level value-binding metadata.
The function-like macro accepts either semicolons or commas between options.

This crate also exports `gpui_form_component::component_shape!` for simple
runtime-local shapes. It is intentionally narrower than the derive crate macro,
so reusable shapes should generally use `gpui_form_derive::component_shape!`.

For component shapes that should participate in generated prototyping
subscriptions, implement `shape::ComponentValueBinding<T>` for the same
shape and either set `value_binding` on the shape metadata or add
`.value_binding()` to the field's shape expression for a single field. The
binding seeds state from the current form value and maps component events
to `FormValueChange<T>`. The binding's associated `Event` type is the actual
event enum emitted by the state. Components that own their state can expose
their own event enum and mark the shape with
`shape::OwnedComponentValueBinding<T>`; external wrappers keep their
upstream event type and convert it in `form_value_change`.

Reusable shapes can also publish `shape::ComponentPrototyping` metadata.
Set `field_suffix = "..."` through `gpui_form_derive::component_shape!`,
the runtime `component_shape!` helper, or `#[gpui_form_shape(...)]` so
prototyping generators can emit names such as `email_input` without deriving
that suffix from the shape type. Generated value-binding scaffolds can use
`ComponentStateOf`,
`ComponentEventOf`, `seed_value_binding_state`, and `form_value_change` to
avoid repeating associated-type projections at every call site.

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the `GpuiForm` facade
- [`gpui-form-component-derive`](../gpui-form-component-derive/README.md) when
  you want only the `InfiniteSelect` derive plus this runtime layer
- [`gpui-component`](https://github.com/longbridge/gpui-component) for the
  upstream date-picker widget and other base components
- [`gpui-form-schema`](../gpui-form-schema/README.md) for metadata and inventory
- [`gpui-form-prototyping-core`](../gpui-form-prototyping-core/README.md) for
  scaffold generation
