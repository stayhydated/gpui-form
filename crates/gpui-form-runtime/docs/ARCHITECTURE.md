# gpui-form-runtime Architecture

`gpui-form-runtime` is the GPUI runtime contract layer for generated forms.

## Purpose

This crate exists so component-shape contracts are not owned by any specific
component collection. It contains the runtime traits and helper types that
generated `#[derive(GpuiForm)]` output references after macro expansion.

## Modules

- `src/lib.rs`: public module surface
- `src/shape.rs`: `ComponentShape`, `DeclaredComponentShape`,
  `ComponentShapeFor`, required-value and value-binding policy markers,
  value-holder storage helpers, value-binding contracts, and generated-code
  helper aliases/functions

## Boundaries

`gpui-form-runtime` may depend on `gpui` because `ComponentShape::new`,
`ComponentValueBinding`, and `ComponentStateValueBinding` use `Window`,
`Context`, and `EventEmitter`. It may also depend on `gpui-form-core` for
UI-neutral helper rules such as shared component suffix validation.

Generated form field checks require `DeclaredComponentShape`, which is emitted
only by `gpui_form_derive::component_shape!` and
`#[derive(gpui_form_derive::ComponentShape)]`. A hand-written
`ComponentShape` impl can exist for lower-level runtime experiments, but it is
not a usable `#[gpui_form(component(...))]` field shape; use the macro or derive
to declare field shapes.

Generated inherited value-binding checks dispatch through
`ComponentShape::ValueBindingPolicy` and `AssertComponentValueBindingPolicy`
rather than const generics. This keeps generic shape paths such as
`Input<T>` type-checkable. Schema and prototyping metadata read the sealed
policy associated types directly so declared shapes cannot make const metadata
disagree with their storage or binding policy.

Generated field type compatibility checks dispatch through
`ComponentShapeFor<Value>`. Shape macros emit implementations only for explicit
`value = ...` / `values(...)` metadata; curated component shapes can provide
narrower manual implementations so their own traits and
`#[diagnostic::on_unimplemented]` notes describe unsupported field value types.

Generated validation asks `ValueStorage::is_present` whether a
policy-owned holder field currently contains a value. `RequiredValueStorage` reports
`None` as missing; `DirectValueStorage` always reports present because its
storage is direct `T`.

`ComponentPrototyping::field_suffix(...)` validates suffixes through
`gpui-form-core`, using the same const ASCII identifier rules as schema
metadata. This keeps shape declarations from publishing invalid prototyping
metadata while preserving the runtime crate's no-schema-dependency boundary.

It should not depend on:

- `gpui-form-component`, which owns concrete reusable widgets
- `gpui-form-collection`, which owns curated shape wrappers around
  `gpui-component`
- `gpui-form-schema`, which owns inventory metadata
- `gpui-form-derive`, which owns proc-macro parsing and expansion
