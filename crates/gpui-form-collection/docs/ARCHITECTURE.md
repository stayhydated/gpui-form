# gpui-form-collection Architecture

`gpui-form-collection` is the curated component-representation crate for
`gpui-form`.

## Purpose

This crate exists to keep framework-specific component choices out of
`gpui-form` internals. The core derive should understand the component contract;
this crate owns reusable component types for common `gpui-component` widgets.

## Boundaries

- `gpui-form` owns the main form derive API and schema metadata.
- `gpui-form-component` owns the custom component contract.
- `gpui-form-collection` owns opt-in components declared through
  `gpui_form_derive::custom_component!` and any value adapters those components need.
- Collection components publish `CustomComponentShape::PROTOTYPING.field_suffix`
  values so prototyping output can use stable widget-role names without
  rediscovering the same suffixes from type names.
- Application crates may define their own component shape types directly when this
  collection is not the right owner.

## Current Components

- `input::Input<T>`: entity-backed text input over
  `gpui_component::input::InputState`, with generic `FromStr`/`ToString` value
  binding.
- `select::Select<T, D = Vec<T>>`: enum-backed select over
  `gpui_component::select::SelectState<D>`, with `IntoEnumIterator`,
  `Default`, `PartialEq`, `SelectItem<Value = T>`, and delegate bounds.
  Codegen specializes `D` to `gpui_component::select::SearchableVec<T>` when a
  field uses `::searchable(true)`.
- `checkbox::Checkbox`: value-bound checkbox wrapper that stores checked
  state in an entity and renders `gpui_component::checkbox::Checkbox`.
- `switch::Switch`: value-bound switch wrapper that stores checked state
  in an entity and renders `gpui_component::switch::Switch`.

## When To Update This Document

Update this file when a new component family is added, when a component changes its
value-binding or prototyping metadata semantics, or when ownership moves between
this collection and a downstream application crate.
