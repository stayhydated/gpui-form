default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt

p-lib-forms:
  cargo run -p prototyping

update_crate_paths:
  crates-paths -c gpui -o crates/gpui-form-core/src/implementations/__crate_paths/gpui.rs
  crates-paths -c gpui-component -o crates/gpui-form-core/src/implementations/__crate_paths/gpui_components.rs
