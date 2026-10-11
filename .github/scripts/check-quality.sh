#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
cargo fmt --all --check
rustfmt --check --edition 2024 .github/feature-smoke.rs
cargo sort-derives --check
taplo fmt --check
rumdl check .
typos
actionlint
shellcheck .github/scripts/*.sh
