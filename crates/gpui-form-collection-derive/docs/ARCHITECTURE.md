# gpui-form-collection-derive Architecture

`gpui-form-collection-derive` owns the derive macro implementation used by
collection-backed enum selects.

## Purpose

This crate exists so collection-oriented trait-generation can evolve independently
from the core `GpuiForm` macro surface.

## Responsibility

- `src/select_item.rs`: expands `#[derive(SelectItem)]` into an implementation
  of `gpui_component::select::SelectItem`.
- It handles enum fallback title synthesis and optional
  `#[select_item(fluent)]` behavior, including variant names for tuple and struct
  variants.

## Boundaries

- The core form derive crate (`gpui-form-derive`) focuses on
  `GpuiForm`, custom component shape macros, and custom shape declarations.
- This crate focuses only on enum-backed select item title/value implementations.
- Runtime shapes and adapters remain in `gpui-form-collection`.
- Applications import `#[derive(SelectItem)]` from this crate explicitly.

## When To Update This Document

Update this file when macro expansion behavior changes, such as new
`select_item(...)` options or a change in generated trait contracts.
