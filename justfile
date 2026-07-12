set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
    rumdl fmt .

clippy:
    cargo clippy --workspace --all-features --exclude some-lib-forms

check:
    cargo check --workspace --all-features --exclude some-lib-forms

test:
    cargo test --workspace --all-features

cov:
    cargo llvm-cov --workspace --all-features --all-targets --exclude gpui-form-component-story --exclude some-lib-forms --exclude prototyping

test-publish:
    cargo publish --workspace --dry-run --allow-dirty
