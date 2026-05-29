# gpui-form-collection Architecture

`gpui-form-collection` is the curated component-representation crate for
`gpui-form`.

## Purpose

This crate exists to keep framework-specific component choices out of
`gpui-form` internals. The core derive should understand the component contract;
this crate owns reusable component types for common `gpui-component` widgets.

## Boundaries

- `gpui-form` owns the main form derive API and schema metadata.
- `gpui-form-runtime` owns the component shape contract.
- `gpui-form-collection` owns opt-in components declared through
  `gpui_form_derive::component_shape!` and any value adapters those components need.
- Collection components publish `ComponentShape::PROTOTYPING.field_suffix`
  values so prototyping output can use stable widget-role names without
  rediscovering the same suffixes from type names.
- Collection components that can synthesize missing values publish
  `ValueStoragePolicy = DirectValueStorage` so consuming form fields inherit
  direct `T` value-holder storage.
- Application crates may define their own component shape types directly when this
  collection is not the right owner.

## Current Components

- `input::Input<T>`: entity-backed text input over
  `gpui_component::input::InputState`, with generic `FromStr`/`ToString` value
  binding.
- `select::Select<T, D = Vec<T>>`: enum-backed select over
  `gpui_component::select::SelectState<D>`, with `IntoEnumIterator`,
  `Default`, `PartialEq`, `SelectItem<Value = T>`, and delegate bounds.
  Applications that need search or other select-specific options should expose
  those choices through a dedicated `ComponentShape` wrapper.
- `combobox::Combobox<T, D = Vec<T>>`: enum-backed combobox over
  `gpui_component::combobox::ComboboxState<D>`, with `IntoEnumIterator`,
  `PartialEq`, `SelectItem<Value = T>`, and delegate bounds. Selection is
  persisted as a `Vec<T>` form value.
- `checkbox::Checkbox`: value-bound checkbox wrapper that stores checked
  state in an entity and renders `gpui_component::checkbox::Checkbox`.
- `switch::Switch`: value-bound switch wrapper that stores checked state
  in an entity and renders `gpui_component::switch::Switch`.
- `number_input::NumberInput<T>`: number input wrapper around
  `gpui_component::input::InputState`, re-emitting normalized change events for
  both keystrokes and step actions from
  `gpui_component::input::NumberInput`.
- `slider::Slider`: value-bound numeric slider backed by
  `gpui_component::slider::SliderState` and `SliderEvent`.
- `color_picker::ColorPicker`: value-bound color picker backed by
  `gpui_component::color_picker::ColorPickerState`.
- `date_picker::DatePicker`: value-bound date picker backed by
  `gpui_component::date_picker::DatePickerState`; `DatePickerEvent::Change`
  events map clear and selected dates to `chrono::NaiveDate` form values.
- `date_picker::DateRangePicker`: range-mode wrapper backed by
  `gpui_component::date_picker::DatePickerState::range`; `Change` events map
  clear and selected ranges to `(chrono::NaiveDate, chrono::NaiveDate)` form
  values.
- `otp_input::OtpInput<T>`: OTP input wrapper backed by
  `gpui_component::input::OtpState`; `InputEvent::Change` maps to parsed value
  updates.

## When To Update This Document

Update this file when a new component family is added, when a component changes its
value-binding or prototyping metadata semantics, or when ownership moves between
this collection and a downstream application crate.
