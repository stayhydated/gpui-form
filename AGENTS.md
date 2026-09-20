# Working in gpui-form

Start in `crates/gpui-form` for application-facing form changes. Use
`just --list` for the workspace command index and `examples/README.md` for
runnable examples. The default Cargo member is the `some-lib-forms` gallery;
select a package with `-p` for focused library work.

## Find the owning surface

Paths below are relative to `crates/` unless stated otherwise.

| Surface | Responsibility |
|---|---|
| `gpui-form` | Public facade, feature flags, generated runtime imports, and end-to-end derive tests |
| `gpui-form-derive` | `GpuiForm` and MCP submit macro expansion |
| `gpui-form-codegen` | Internal component parsing, crate-path resolution, and metadata token generation |
| `gpui-form-core` | UI-neutral field identities, suffix checks, and numeric validation |
| `gpui-form-schema` | Public schema, inventory, and resolved generator metadata |
| `gpui-form-runtime` | Public shape storage policy and value-binding contracts |
| `gpui-form-collection` and `gpui-form-collection-derive` | Built-in form shapes and `SelectItem` derive |
| `gpui-form-component` and `gpui-form-component-derive` | Date/file pickers, infinite selects, and `InfiniteSelect` derive |
| `gpui-form-mcp` | Submit handlers, editor sessions, schemas, resources, and registration |
| `gpui-form-prototyping-core` | Scaffold generation from `GpuiFormShape` inventory |
| `gpui-form-component-story` | Interactive component gallery and its localization assets |

At the workspace root:

- `examples/some-lib` owns example domain models and their Fluent resources.
- `examples/prototyping` generates the form views in
  `examples/some-lib-forms/src/forms` and `examples/prototyping/output`.
- `examples/some-lib-forms` hosts the native and WebAssembly form gallery.
- `examples/mcp-submit` demonstrates a stdio form server.
- `book/src` contains the user guide; start with `SUMMARY.md`.
- `skills/use-gpui-form` covers application usage;
  `skills/use-gpui-form-component-shapes` covers custom shape integration.
- `web/src/lib.rs` owns the public catalog and destinations; `xtask` owns the
  book, language-model documentation, demo, and site build commands.

## Keep related surfaces aligned

- When public attributes, features, generated imports, or conversion behavior
  change, update the affected root/crate READMEs, `book/src`, public rustdocs,
  repository skills, and consuming examples. Keep README usage concise; use
  CI, Codecov, book, and crates.io badges for their destinations.
- For derive behavior and diagnostics, update the facade's tests in
  `crates/gpui-form/tests`, including `tests/ui` fixtures and `.stderr`
  expectations. Macro implementation tests live beside the derive code.
- When shape metadata changes, align `gpui-form-codegen`, `gpui-form-schema`,
  `gpui-form-runtime`, and the consumers in `gpui-form-prototyping-core` and
  `gpui-form-mcp`. Update the shape-owning crate and applicable skills too.
- When scaffold output changes, edit the generator or inventory first, run
  `cargo run -p prototyping`, and review both generated output directories.
  Keep token snapshots under
  `crates/gpui-form-prototyping-core/src/implementations/snapshots` aligned.
- For localization changes in `examples/some-lib`,
  `crates/gpui-form-component`, or `crates/gpui-form-component-story`, align the
  package's `i18n.toml`, `i18n/` resources, typed messages, and usage examples.
- Build publication assets under `web/public` through `cargo xtask`; edit
  `book/src`, `web/src`, or the gallery sources that produce them.

Use source-adjacent rustdocs, implementation comments, tests, and snapshots to
understand internal behavior before changing its public description.

## Validate the changed surface

Use the smallest applicable check from the repository workflows:

| Change | Check |
|---|---|
| Library behavior | `cargo test -p <owning-package>` |
| Generated form API or diagnostics | `cargo test -p gpui-form --test ui` |
| Holder conversion | `cargo test -p gpui-form --test holder_conversion` |
| Scaffold generation | `cargo test -p gpui-form-prototyping-core` and `cargo run -p prototyping` |
| Markdown | `rumdl check <changed-paths>` |
| User guide | `cargo xtask build book` and `cargo xtask build llms-txt` |
| API documentation | `cargo doc --workspace --all-features --no-deps --locked` |
| Full publication pipeline | `just web-build` |

For book validation, set `MDBOOK_BUILD__CREATE_MISSING=false` so missing
navigation targets are reported without creating chapters. For broader Rust
changes, use the matching `justfile` recipes; `just fmt` rewrites files.
CI runs tests with all features across Linux, macOS, and Windows.

Report checks that ran, failures, and checks skipped with their reasons. When
outputs change, state whether they were regenerated from their owning sources.
