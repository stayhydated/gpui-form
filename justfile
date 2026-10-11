set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
    rumdl fmt .

quality:
    cargo fmt --all --check
    rustfmt --check --edition 2024 .github/feature-smoke.rs
    cargo sort-derives --check
    taplo fmt --check
    rumdl check .
    typos
    actionlint
    git ls-files -z '*.sh' '*.bash' | xargs -0 --no-run-if-empty shellcheck

clippy:
    cargo clippy --workspace --all-features --all-targets --locked --exclude some-lib-forms -- -D warnings

check:
    cargo check --workspace --all-features --locked --exclude some-lib-forms

test:
    cargo test --workspace --all-features --locked

cov:
    cargo llvm-cov --workspace --all-features --all-targets \
        --exclude gpui-form-component-story \
        --exclude prototyping \
        --exclude some-lib-forms \
        --exclude xtask \
        --exclude web

test-publish:
    cargo publish --workspace --dry-run --locked --allow-dirty

book:
    mdbook serve book

gpui-demo-build:
    cargo xtask build gpui-demo

web-build: gpui-demo-build
    cargo xtask build book
    cargo xtask build llms-txt
    cargo xtask build web

web: web-build
    dx serve --package web

web-preview: web-build
    cargo xtask preview web
