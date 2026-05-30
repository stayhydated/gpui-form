# gpui-form-component-derive Architecture

`gpui-form-component-derive` owns the proc-macro entry points that are specific
to the runtime helper layer in `gpui-form-component`.

## Entry Points

`src/lib.rs` defines:

- `#[derive(InfiniteSelect)]`

## Module Layout

- `src/infinite_select.rs`: `InfiniteSelect` expansion

## `InfiniteSelect`

This derive emits an implementation of the runtime crate's
`InfiniteSelect` trait for nested enums.

Responsibilities:

- parse unit, single-field tuple, and single-field struct variants
- support `#[tuple_enum(skip)]` for variants that should not appear in the
  select tree
- support `#[tuple_enum(key = "...")]` for persisted key overrides
- support `#[fluent_kv(keys = ["label", "description"], keys_this)]` metadata
  while allowing sibling `es-fluent` derives to route those messages through
  their own `#[fluent(namespace = "...")]` metadata
- emit recursive child traversal methods that match the runtime contract in
  `gpui-form-component`
- keep `fluent_kv` label/description metadata available to callers with an
  explicit localizer; generated runtime trait methods use fallback names
  because the contract is localizer-free
- validate that persisted keys stay unique within one enum

## Dependency Role

This crate should stay focused on the infinite-select runtime contract.

It should not own:

- `GpuiForm` struct parsing and token generation
- select-item derives unrelated to infinite-select
- schema metadata or inventory registration

Those belong in `gpui-form-derive`, `gpui-form-codegen`, and
`gpui-form-schema`.

## Data Flow

1. A user derives `InfiniteSelect` through `gpui-form-component` with the
   `derive` feature or directly through this crate.
1. The macro resolves the runtime crate path through
   `gpui-form-codegen::resolve_crate_path` and emits trait impls against
   `gpui-form-component`.
1. Users therefore depend on `gpui-form-component` explicitly whenever they use
   this derive.
1. Runtime code in `gpui-form-component` consumes that impl through
   `InfiniteSelectItem`, `InfiniteSelectPath`, `InfiniteSelectKeyPath`, and the
   path reconstruction helpers.
1. `GpuiForm` and prototyping output can then build cascading selects on top of
   those runtime helpers.

## When To Update This Document

Update this file when:

- the `InfiniteSelect` derive responsibilities change
- expansion output targets a different runtime contract
- component-oriented proc macros move into or out of this crate
