# AGENTS.md

This is the working guide for contributors and coding agents in the
`gpui-form` workspace.

Use it to decide:

1. which crate or example owns a change,
2. which docs, rustdocs, book pages, skills, examples, or generated outputs
   must change with it,
3. which narrow validation command proves the edit.

For most application-facing form work, start in `crates/gpui-form`. Use
`crates/gpui-form-prototyping-core` for GPUI scaffolding from `GpuiFormShape`
inventory data. Check `just --list` before broad validation.
Use `book/src/SUMMARY.md` to route user-guide edits and `web/src/lib.rs` for
the public catalog.

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
2. Treat README files, `book/src`, `examples/README.md`, and in-repository
   `skills/*` guidance as user-facing.
3. Treat `//!` and `///` rustdocs, source-adjacent comments, tests, snapshots,
   and examples as the internal behavior record.
4. Update every public surface that describes a changed derive attribute,
   component contract, feature flag, runtime import, prototyping workflow, MCP
   workflow, or supported example.
5. Run the smallest command that proves the edited behavior or docs surface.

Route public documentation to the README, example, skill, and rustdoc surfaces
named below. Keep implementation rationale next to the relevant module as
rustdocs/source comments, in focused tests or snapshots, or in this guide when
it affects agent routing.

## Documentation Sync

When public usage changes, update the applicable set:

- root `README.md`
- `book/src` for task-oriented guides and the published chapter structure
- `crates/gpui-form/README.md`
- affected crate `README.md` files
- `examples/README.md` and showcased example crates
- `skills/use-gpui-form` and `skills/use-gpui-form-component-shapes`
- rustdocs on public traits, structs, macros, and helper functions
- `web/src/lib.rs` when the catalog description or published destinations
  change

Keep these specific surfaces aligned:

- root `README.md` and `crates/gpui-form/README.md` for installation,
  quick-start, feature flags, runtime imports, MCP, prototyping, and examples
- root `README.md`, affected crate READMEs, and matching `book/src` chapters
  for public workflows covered by the guide
- root `README.md`, `crates/gpui-form-derive/README.md`, and
  `skills/use-gpui-form/references/api-map.md` for supported component syntax
  and derive attributes
- root `README.md`, `crates/gpui-form-prototyping-core/README.md`,
  `examples/README.md`, and `examples/prototyping` for inventory/codegen
  workflows
- `examples/README.md` as the canonical index for runnable workspace examples
- package-local `i18n.toml`, `i18n/` Fluent resources, `src/i18n.rs`, and
  matching README/example text for localization changes in `examples/some-lib`,
  `crates/gpui-form-component`, or `crates/gpui-form-component-story`

The checked-in sources are `book/src`, `web/src`, and
`examples/some-lib-forms`. Build `web/public/book`, `web/public/llms*`,
`web/public/gpui-demo`, and `web/dist` through `cargo xtask`; do not maintain
those generated publication artifacts by hand.

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
  Shared native and WebAssembly Storybook gallery for generated forms. The
  `demo` example target used by the public site also launches natively and runs
  the same registrations and gallery startup as the native binary.
  Run with `cargo run -p some-lib-forms`.

- `examples/prototyping`
  Generator that reads `GpuiFormShape` inventory data and emits form
  scaffolding into `examples/some-lib-forms/src/forms` and
  `examples/prototyping/output`.
  Run with `cargo run -p prototyping`.

- `examples/mcp-submit`
  Stdio MCP server that exposes generated form value holders as MCP tools.
  Run with `cargo run -p mcp-submit`.

- `crates/gpui-form-component-story`
  Storybook-style GPUI app for reusable runtime components.
  Run with `cargo run -p gpui-form-component-story`.

### Documentation, Demo, and Publishing

- `book/src`
  Audience: user-facing.
  Role: mdBook source for installation, field intent, validation, component
  shapes, MCP, and prototyping workflows.

- `examples/some-lib-forms/examples/demo.rs`
  Audience: user-facing.
  Role: native and nightly Trunk entry point for the full `some-lib-forms`
  Storybook gallery.

- `web`
  Audience: user-facing.
  Role: Dioxus catalog that links the book, GPUI demo, API docs, and source.

- `xtask`
  Audience: internal.
  Role: reproducible book, `llms.txt`, GPUI demo, and Pages-site builds.

## Editing Rules

When editing Rust crates:

- Use `cargo` for focused build, test, and run tasks. Use `justfile` recipes
  for workspace-wide format, clippy, check, test, coverage, and dry-run publish
  tasks.
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
- Keep `crates/gpui-form-prototyping-core/src/implementations/snapshots`
  aligned when generator token output changes.

When writing tests:

- Prefer focused crate-level tests near the changed subsystem.
- For macro or token-generation changes, test emitted behavior at the derive
  and codegen boundary rather than only the lowest-level helper.

## Validation

Run the narrowest command that proves the edit. CI also runs fmt, clippy,
workspace tests, docs, package-content listing, cargo-machete, and an es-fluent
FTL check.

- `cargo check -p gpui-form` for facade compile checks, or the same `-p` form
  for the package that owns a focused edit
- `cargo test -p gpui-form` for facade behavior changes, or the same `-p` form
  for the package that owns a focused edit
- `cargo test -p gpui-form-derive --test ui` for derive UI diagnostics
- `cargo test -p gpui-form-prototyping-core` for prototyping generator or
  snapshot changes
- `cargo run -p prototyping` after generator/inventory output changes
- `cargo xtask build book` and `cargo xtask build llms-txt` for book changes
- `cargo xtask build gpui-demo` for the nightly Wasm GPUI example
- `cargo xtask build web` after the book, language-model docs, and demo assets
  exist, or `just web-build` for the complete publication pipeline
- `cargo doc --workspace --all-features --no-deps --locked` when matching the CI
  docs job
- `cargo package --workspace --list` when matching the CI package job
- `just cov` for LLVM source coverage across publishable library crates; the
  recipe also exercises the headless example and MCP integration packages,
  while excluding GUI applications, prototyping, and publication tooling
- `just fmt`, `just clippy`, `just check`, `just test`, `just cov`, or the
  matching `justfile` recipe when a change spans each recipe's scope

CI generates a Cobertura report with `cargo-llvm-cov` and publishes it to
Codecov using the same crate scope as `just cov`.

If validation cannot be run, state why and what remains unvalidated. Do not
claim a change works or was validated unless a proving command was run; for
generated output, also state whether it was regenerated from source metadata.
