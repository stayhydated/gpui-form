# gpui-form

[![CI][ci-badge]][ci]
[![Codecov][codecov-badge]][codecov]
[![Book][book-badge]][book]
[![crates.io: gpui-form][gpui-form-badge]][gpui-form-crate]

`gpui-form` generates typed form state for [GPUI Kit][gpui-kit] applications.
Application developers annotate model fields once, then use generated holders
and components to render, validate, and reconstruct those models.

All workspace packages require Rust 1.99 or newer. GPUI integrations use
GPUI Kit 0.7; headless MCP consumers can disable the facade's `runtime` feature.

## Overview

The `GpuiForm` derive generates typed field identities, GPUI entity storage,
component constructors, and a form value holder. Every field uses exactly one
`component(...)`, `hidden`, or `skip` intent.

## Crates

| Crate | Purpose |
| --- | --- |
| [`gpui-form`][gpui-form-readme] | Application facade for derives, generated state, schema access, and optional MCP tools. |
| [`gpui-form-codegen`][gpui-form-codegen-readme] | Parsing and token-generation helpers for macro implementations. |
| [`gpui-form-collection`][gpui-form-collection-readme] | Ready-made form shapes for common GPUI Kit controls. |
| [`gpui-form-collection-derive`][gpui-form-collection-derive-readme] | `SelectItem` derive for enum choices used by collection shapes. |
| [`gpui-form-component`][gpui-form-component-readme] | Localized date/file pickers, cascading infinite selects, and controlled ordered row editors. |
| [`gpui-form-component-derive`][gpui-form-component-derive-readme] | `InfiniteSelect` derive, normally re-exported by `gpui-form-component`. |
| [`gpui-form-component-story`][gpui-form-component-story-readme] | Internal interactive component gallery and localization examples. |
| [`gpui-form-core`][gpui-form-core-readme] | UI-neutral field identities, suffix checks, and numeric editing validation. |
| [`gpui-form-derive`][gpui-form-derive-readme] | Procedural macros, normally re-exported by the facade. |
| [`gpui-form-mcp`][gpui-form-mcp-readme] | Experimental submit handlers, metadata, and headless editor sessions. |
| [`gpui-form-prototyping-core`][gpui-form-prototyping-core-readme] | Generate application-owned GPUI form scaffolds from inventory metadata. |
| [`gpui-form-runtime`][gpui-form-runtime-readme] | Storage and value-binding contracts for reusable component shapes. |
| [`gpui-form-schema`][gpui-form-schema-readme] | Form inventory and validated metadata for generators and integrations. |

Most applications start with [`gpui-form`][gpui-form-readme] and add only the
component crates they use. The [workspace examples][examples] demonstrate complete form views,
component galleries, scaffold generation, and MCP servers.

[ci-badge]: https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg?branch=master
[ci]: https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml
[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[book-badge]: https://img.shields.io/badge/Book-mdBook-blue
[book]: https://stayhydated.github.io/gpui-form/book/
[gpui-form-badge]: https://img.shields.io/crates/v/gpui-form.svg?label=gpui-form
[gpui-form-crate]: https://crates.io/crates/gpui-form
[gpui-kit]: https://github.com/longbridge/gpui-kit
[gpui-form-readme]: crates/gpui-form/README.md
[examples]: examples/README.md
[gpui-form-codegen-readme]: crates/gpui-form-codegen/README.md
[gpui-form-collection-readme]: crates/gpui-form-collection/README.md
[gpui-form-collection-derive-readme]: crates/gpui-form-collection-derive/README.md
[gpui-form-component-readme]: crates/gpui-form-component/README.md
[gpui-form-component-derive-readme]: crates/gpui-form-component-derive/README.md
[gpui-form-component-story-readme]: crates/gpui-form-component-story/README.md
[gpui-form-core-readme]: crates/gpui-form-core/README.md
[gpui-form-derive-readme]: crates/gpui-form-derive/README.md
[gpui-form-mcp-readme]: crates/gpui-form-mcp/README.md
[gpui-form-prototyping-core-readme]: crates/gpui-form-prototyping-core/README.md
[gpui-form-runtime-readme]: crates/gpui-form-runtime/README.md
[gpui-form-schema-readme]: crates/gpui-form-schema/README.md
