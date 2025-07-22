default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt

p-lib-forms:
  cargo run -p prototyping
