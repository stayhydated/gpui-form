# AGENTS.md

This is the working guide for contributors and coding agents in the
`gpui-form` workspace.

Use it to decide:

1. where documentation belongs,
2. whether a crate or surface is user-facing, public integration, or internal,
3. which related docs, examples, and skills must change together,
4. which validation command should run before handoff.

For most application code, start with `crates/gpui-form`.

Reach for `crates/gpui-form-prototyping-core` when you want to generate GPUI
scaffolding from `GpuiFormShape` inventory data instead of wiring forms by hand.

## Project Summary

`gpui-form` is a Rust form-generation ecosystem built on top of `gpui` and
`gpui-component`, centered on `#[derive(GpuiForm)]`.

Its priorities are:

1. **Type safety**: generate strongly typed form state, metadata, and helper APIs at compile time.
2. **Ergonomics**: keep `#[derive(GpuiForm)]` and related attributes concise enough for normal application structs.
3. **Developer experience**: support inventory-driven prototyping, component shapes, and layered crates that can be used directly when needed.

## Quick Decision Flow

Before editing, classify the change:

1. **Find the surface in the workspace map.** Use its audience label to decide
   how much public explanation the change needs.
2. **Place documentation by content, not by crate audience.** README files are
   always user-facing. Internal design belongs in the matching
   `docs/ARCHITECTURE.md`.
3. **Sync public workflow changes.** If derive attributes, supported component
   behavior, Koruma validation wiring, runtime imports, component shapes,
   prototyping output, or recommended usage changes, update the relevant
   README, example, architecture note, and `.agents/skills/*` guidance in the
   same change when applicable.
4. **Validate narrowly.** Run the smallest command that proves the edited
   behavior or documentation surface is still sound.

## Audience Labels

These labels describe the crate or surface itself, not the documentation file
being edited:

- **User-facing**: normal entry points for application developers.
- **Public integration**: public crates meant for extensions, lower-level runtime access, tooling, or deeper customization. These are usually not the default starting point.
- **Internal**: workspace plumbing, parse-time and token-generation internals, examples, and maintenance surfaces.

## Documentation Placement

### User-Facing Documentation

Treat these surfaces as user-facing:

- the root `README.md`,
- `examples/README.md`,
- crate-level `README.md` files under `crates/`.

Even README files for public-integration or internal crates should explain:

- who the crate is for,
- what it does,
- what most users should use instead.

Keep user-facing documentation example-first. Prefer Rust snippets over
prose-only explanations when showing behavior changes.

### Internal Documentation

Use the relevant `docs/ARCHITECTURE.md` file for internal documentation, such
as the crate-level paths listed in the workspace map.

Keep these topics in architecture documents, not in READMEs:

- implementation details,
- macro expansion and parse-time behavior,
- subsystem boundaries,
- data flow between facade, derive, runtime, schema, and prototyping layers,
- design rationale,
- internal relationships.

### Skill Guidance

`.agents/skills/use-gpui-form` is hosted in this repository as public
`gpui-form` usage guidance for application developers. It is not internal
architecture, maintenance, CI, release, or contributor-only workflow
documentation.

Update relevant in-repository `.agents/skills/*` guidance when a code change
alters user-facing workflows, derive attributes, generated output, component
syntax, runtime integration patterns, prototyping patterns, or recommended usage.

## Synchronization Rules

When a substantive change modifies a public derive attribute, supported
component set, Koruma validation wiring, runtime import, component shape
contract, prototyping workflow, or other user-visible API shape:

1. Update the root `README.md`.
2. Update `crates/gpui-form/README.md`.
3. Update the affected crate `README.md` files.
4. Update `examples/README.md` and the relevant example crates when showcased behavior changes.
5. Update relevant in-repository `.agents/skills/*` guidance.
6. Update the matching crate `docs/ARCHITECTURE.md` when boundaries or behavior change.
7. Keep these surfaces aligned in the same change unless there is a documented reason not to.

`examples/README.md` is the canonical index for runnable workspace examples.

Keep the root `README.md` and `crates/gpui-form/README.md` aligned for install,
quick-start, feature, and runtime import guidance.

Keep supported-component docs aligned across the root `README.md` and
`crates/gpui-form-derive/README.md`.

Keep prototyping docs aligned across the root `README.md`,
`crates/gpui-form-prototyping-core/README.md`, and `examples/prototyping` when
inventory or codegen workflows change.

## Workspace Map

### Main User-Facing Entry Points

- `crates/gpui-form`
  Audience: **User-facing**
  Docs: [Architecture](crates/gpui-form/docs/ARCHITECTURE.md)
  Role: workspace facade, default entry point, and home of the public feature flags. Re-exports `GpuiForm` plus `core`, `schema`, and `bon`; numeric helpers are available under `gpui_form::core::numeric`. Component shape runtime contracts, component runtimes, collection shapes, and their derives are imported explicitly from their own crates.

### Public Integration Crates

- `crates/gpui-form-core`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-form-core/docs/ARCHITECTURE.md)
  Role: pure, non-GPUI helper logic such as numeric validation. Most application users should start with `gpui-form`.

- `crates/gpui-form-collection`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-form-collection/docs/ARCHITECTURE.md)
  Role: curated component shapes and value-binding adapters for common `gpui-component` widgets used by generated forms and integration layers.

- `crates/gpui-form-collection-derive`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-form-collection-derive/docs/ARCHITECTURE.md)
  Role: proc macros for collection runtime integration and derive-time helpers that pair with `gpui-form-collection`.

- `crates/gpui-form-runtime`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-form-runtime/docs/ARCHITECTURE.md)
  Role: GPUI-facing runtime contracts referenced by generated form code, including `ComponentShape` and value-binding helpers. Users add this explicitly when they use component-backed fields or custom component shapes.

- `crates/gpui-form-component`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-form-component/docs/ARCHITECTURE.md)
  Role: GPUI-facing runtime implementations for infinite select, date picker, and file picker. Users import this crate explicitly when they need concrete component runtime APIs.

- `crates/gpui-form-component-derive`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-form-component-derive/docs/ARCHITECTURE.md)
  Role: proc macros for the `InfiniteSelect` runtime surface. Most users should access this through `gpui-form` or `gpui-form-component`.

- `crates/gpui-form-schema`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-form-schema/docs/ARCHITECTURE.md)
  Role: schema metadata, component behavior metadata, and inventory registry types used by derives and prototyping. Most application users should not need it directly unless they are extending metadata or tooling.

- `crates/gpui-form-derive`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-form-derive/docs/ARCHITECTURE.md)
  Role: proc macros for `#[derive(GpuiForm)]`, `#[derive(ComponentShape)]`, `component_shape!`, and `component_value_binding`. Most users should depend on `gpui-form` rather than this crate directly.

- `crates/gpui-form-prototyping-core`
  Audience: **Public integration**
  Docs: [Architecture](crates/gpui-form-prototyping-core/docs/ARCHITECTURE.md)
  Role: prototyping and code-generation helpers that consume `GpuiFormShape` inventory data and generate scaffolded form code.

### Internal Crates

- `crates/gpui-form-codegen`
  Audience: **Internal**
  Docs: [Architecture](crates/gpui-form-codegen/docs/ARCHITECTURE.md)
  Role: parse-time component parsing, per-component layout emission, and token generation used by `gpui-form-derive`.

- `crates/gpui-form-component-story`
  Audience: **Internal**
  Role: Storybook launcher, story UI, and story-only localization assets for the runtime components in `gpui-form-component`.

### Examples and Shared Surfaces

- `examples/README.md`
  Canonical index of runnable workspace examples.

- `examples/i18n`
  Shared localization assets used by the example crates.

- `examples/some-lib`
  Shared example domain types and source structs that derive `GpuiForm`.

- `examples/some-lib-forms`
  Storybook-like GPUI example app for browsing generated forms.

  Run with `cargo run -p some-lib-forms`.

- `crates/gpui-form-component-story`
  Storybook-like GPUI example app for browsing the reusable runtime components.

  Run with `cargo run -p gpui-form-component-story`.

- `examples/prototyping`
  Prototype generator that reads `GpuiFormShape` inventory data and emits form scaffolding.

  Run with `cargo run -p prototyping`.

## Validation and Editing Rules

### Validation After Changes

- Validation is the default after code or workflow changes.
- Run the narrowest command that proves the edited behavior works for the
  affected crate, docs, example, or generated surface.
- Prefer targeted crate, example, docs, or generator checks before full-workspace validation.
- Use `just check`, `just test`, or a more specific `justfile` recipe when the change spans multiple surfaces.
- If validation cannot be run, state why and what remains unvalidated.
- Do not claim a change works unless it was validated, generated from a source of truth, or the remaining risk is explicitly documented.

### When Editing Docs

- Keep READMEs user-facing.
- Move macro expansion details, parser internals, and subsystem design into `docs/ARCHITECTURE.md`.
- Prefer examples over prose-only explanations.
- Sync the root `README.md`, `crates/gpui-form/README.md`, `examples/README.md`, and `.agents/skills/*` guidance when the primary workflow changes.

### When Editing Rust Crates

- Use `cargo` for build, test, and run tasks.
- Keep shared package metadata and dependency versions in the workspace root `Cargo.toml`.
- Prefer `workspace = true` for shared dependencies in workspace crates.
- Use local `path` dependencies only where the workspace already relies on them, mainly in the workspace root and example crates.
- Treat `crates/gpui-form` as the public compatibility boundary unless you are intentionally changing lower-level crate APIs too.

### When Editing Components or Generated Form Metadata

- When adding or changing a component, update `gpui-form-codegen` component parsing and layout, `gpui-form-schema` runtime behavior metadata, and `gpui-form-prototyping-core` `FieldCodeGenerator` mapping together.
- Update user-facing docs for supported components and usage syntax in the same change.
- Keep facade imports and lower-level runtime and type surfaces aligned when generated code paths change.

### When Editing Prototyping or Generated Outputs

- Prefer changing the source generator or inventory metadata over hand-editing generated output.
- Keep `examples/prototyping` aligned with `gpui-form-prototyping-core` when shape metadata or emitted layout changes.

### When Writing Tests

- Prefer focused crate-level tests near the changed subsystem.
- For macro or token-generation changes, test emitted behavior at the derive and codegen boundary rather than only the lowest-level helper.
