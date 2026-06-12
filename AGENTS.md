# AGENTS.md

This is the working guide for contributors and coding agents in the
`gpui-form` workspace.

Use it to decide:

1. which crate or example owns a change,
2. which docs, rustdocs, skills, examples, or generated outputs must change
   with it,
3. which narrow validation command proves the edit.

For most application-facing form work, start in `crates/gpui-form`. Use
`crates/gpui-form-prototyping-core` when generating GPUI scaffolding from
`GpuiFormShape` inventory data.

## Project Summary

`gpui-form` is a Rust form-generation ecosystem built on top of `gpui` and
`gpui-component`, centered on `#[derive(GpuiForm)]`.

Its priorities are:

1. type-safe generated form state, metadata, value holders, and helper APIs,
2. concise derive attributes for normal application structs,
3. inventory-driven prototyping, component shapes, MCP submit integration, and
   layered crates that can be used directly when needed.

## Quick Decision Flow

Before editing:

1. Find the owning surface in the workspace map.
2. Treat README files, `examples/README.md`, and in-repository `skills/*`
   guidance as user-facing.
3. Treat `//!` and `///` rustdocs, source-adjacent comments, tests, snapshots,
   and examples as the internal behavior record.
4. Update every public surface that describes a changed derive attribute,
   component contract, feature flag, runtime import, prototyping workflow, MCP
   workflow, or supported example.
5. Run the smallest command that proves the edited behavior or docs surface.

There is no mdBook/book surface in this workspace. Do not route documentation
updates to a book unless one is added.

Do not add separate architecture markdown files. Internal design notes belong
next to the relevant module as rustdocs/source comments, in focused tests, or
in this guide when they affect agent routing.

## Documentation Sync

When public usage changes, update the applicable set:

- root `README.md`
- `crates/gpui-form/README.md`
- affected crate `README.md` files
- `examples/README.md` and showcased example crates
- `skills/use-gpui-form` and `skills/use-gpui-form-component-shapes`
- rustdocs on public traits, structs, macros, and helper functions

Keep these specific surfaces aligned:

- root `README.md` and `crates/gpui-form/README.md` for installation,
  quick-start, feature flags, runtime imports, MCP, prototyping, and examples
- root `README.md`, `crates/gpui-form-derive/README.md`, and
  `skills/use-gpui-form/references/api-map.md` for supported component syntax
  and derive attributes
- root `README.md`, `crates/gpui-form-prototyping-core/README.md`,
  `examples/README.md`, and `examples/prototyping` for inventory/codegen
  workflows
- `examples/README.md` as the canonical index for runnable workspace examples

## Workspace Map

### Main User-Facing Entry Point

- `crates/gpui-form`
  Audience: user-facing.
  Role: facade, default entry point, and home of public feature flags. It
  re-exports `GpuiForm`, `core`, `runtime`, `schema`, and `bon`; `mcp` and
  `mcp_submit` are available behind the experimental `mcp` feature.

### Public Integration Crates

- `crates/gpui-form-core`
  Role: non-GPUI helpers such as numeric validation and component suffix
  validation.

- `crates/gpui-form-collection`
  Role: curated component shapes and value bindings for common
  `gpui-component` widgets.

- `crates/gpui-form-collection-derive`
  Role: proc macros such as `SelectItem` that pair with
  `gpui-form-collection`.

- `crates/gpui-form-runtime`
  Role: GPUI-facing runtime contracts used by generated form code, including
  component-shape storage and value-binding helpers.

- `crates/gpui-form-component`
  Role: runtime implementations for infinite select, date/date-range picker,
  and file picker, plus optional built-in component-shape impls.

- `crates/gpui-form-component-derive`
  Role: `InfiniteSelect` derive macro for the runtime surface in
  `gpui-form-component`.

- `crates/gpui-form-schema`
  Role: schema metadata, component behavior metadata, and inventory registry
  types used by derives, MCP, and prototyping.

- `crates/gpui-form-derive`
  Role: proc macro for `#[derive(GpuiForm)]` and `#[gpui_form::mcp_submit]`.
  Most users should depend on `gpui-form` rather than this crate directly.

- `crates/gpui-form-mcp`
  Role: experimental MCP submit/edit integration for generated form value
  holders, including schema generation, typed handler registration, inventory
  registration, and stdio serving.

- `crates/gpui-form-prototyping-core`
  Role: code-generation helpers that consume `GpuiFormShape` inventory data and
  generate scaffolded GPUI form code.

### Internal Crates

- `crates/gpui-form-codegen`
  Role: parse-time component parsing, crate-path resolution, metadata token
  lowering, and component field IR used by derive crates.

- `crates/gpui-form-component-story`
  Role: Storybook-style GPUI app and story-only localization assets for
  reusable runtime components.

### Examples

- `examples/some-lib`
  Shared example domain types and structs that derive `GpuiForm`, plus
  package-local `es-fluent` config and Fluent assets.

- `examples/some-lib-forms`
  Storybook-style GPUI app for generated forms.
  Run with `cargo run -p some-lib-forms`.

- `examples/prototyping`
  Generator that reads `GpuiFormShape` inventory data and emits form
  scaffolding into `examples/some-lib-forms/src/forms` and
  `examples/prototyping/output`.
  Run with `cargo run -p prototyping`.

- `examples/mcp-submit`
  Stdio MCP server that exposes generated form value holders as MCP tools.
  Run with `cargo run -p mcp-submit`.

- `examples/mcp-form-table`
  Headless composed MCP server for a generated form submit tool plus
  `gpui-table` filter arguments.
  Run with `cargo run -p mcp-form-table`.

- `crates/gpui-form-component-story`
  Storybook-style GPUI app for reusable runtime components.
  Run with `cargo run -p gpui-form-component-story`.

## Editing Rules

When editing Rust crates:

- Use `cargo` for build, test, and run tasks.
- Keep shared package metadata and dependency versions in the workspace root
  `Cargo.toml`.
- Prefer `workspace = true` for shared dependencies in workspace crates.
- Treat `crates/gpui-form` as the public facade boundary unless intentionally
  changing lower-level crate APIs.

When adding or changing a component shape:

- Keep `gpui-form-codegen` parsing and metadata emission,
  `gpui-form-schema` metadata, and `gpui-form-prototyping-core` field
  generation aligned.
- Update supported-component docs in the root README,
  `crates/gpui-form/README.md`, `crates/gpui-form-derive/README.md`, affected
  crate READMEs, and public skills.
- Keep facade imports and lower-level runtime/type surfaces aligned when
  generated code paths change.

When editing prototyping or generated outputs:

- Prefer changing the generator or inventory metadata over hand-editing
  generated output.
- Keep `examples/prototyping`, `examples/prototyping/output`, and
  `examples/some-lib-forms/src/forms` aligned.

When writing tests:

- Prefer focused crate-level tests near the changed subsystem.
- For macro or token-generation changes, test emitted behavior at the derive
  and codegen boundary rather than only the lowest-level helper.

## Validation

Validation is the default after code or workflow changes. Run the narrowest
command that proves the edited behavior or docs surface:

- `cargo check -p <package>` for a focused compile check
- `cargo test -p <package>` for focused behavior changes
- `cargo test -p gpui-form-derive --test ui` for derive UI diagnostics
- `cargo run -p prototyping` after generator/inventory output changes
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps`
  after rustdoc or public API documentation changes
- `just check`, `just test`, or the matching `justfile` recipe when a change
  spans multiple surfaces

If validation cannot be run, state why and what remains unvalidated. Do not
claim a change works unless it was validated, generated from source-of-truth
metadata, or the remaining risk is explicitly documented.
