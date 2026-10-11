#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
cargo run -p prototyping --locked
outputs=(examples/prototyping/output examples/some-lib-forms/src/forms)
git diff --exit-code -- "${outputs[@]}"
# git diff alone does not detect newly generated, untracked files.
test -z "$(git status --porcelain --untracked-files=all -- "${outputs[@]}")"
# A second generation also checks determinism within the same checkout.
cargo run -p prototyping --locked
git diff --exit-code -- "${outputs[@]}"
test -z "$(git status --porcelain --untracked-files=all -- "${outputs[@]}")"
