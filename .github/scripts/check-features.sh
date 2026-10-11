#!/usr/bin/env bash
set -euo pipefail
repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"
consumer_dir=$(mktemp -d)
trap 'rm -rf "$consumer_dir"' EXIT
mkdir "$consumer_dir/src"
cp .github/feature-smoke.rs "$consumer_dir/src/main.rs"
# Populate the cache from the committed lock before resolving isolated consumers.
cargo fetch --locked

check() {
    local name=$1 defaults=$2 dependency_features=$3 probe_features=$4
    local feature_list=''
    if [[ -n "$dependency_features" ]]; then
        feature_list="\"${dependency_features//,/\", \"}\""
    fi
    cat > "$consumer_dir/Cargo.toml" <<EOF
[package]
name = "gpui-form-feature-consumer"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[features]
derive = []
runtime = []
mcp = []
chrono = ["dep:chrono"]
rust_decimal = ["dep:rust_decimal"]

[dependencies]
gpui-form = { path = "$repo_root/crates/gpui-form", default-features = $defaults, features = [$feature_list] }
chrono = { version = "0.4", optional = true }
rust_decimal = { version = "1.43", optional = true }
EOF
    # Seed each independent graph with workspace versions. Metadata only adjusts
    # the root package and prunes unused dependencies; offline mode prevents upgrades.
    cp Cargo.lock "$consumer_dir/Cargo.lock"
    cargo metadata --manifest-path "$consumer_dir/Cargo.toml" --offline --format-version 1 > /dev/null
    echo "Checking isolated $name consumer"
    local args=(--manifest-path "$consumer_dir/Cargo.toml" --target-dir "$repo_root/target/feature-consumers" --locked)
    if [[ -n "$probe_features" ]]; then
        args+=(--features "$probe_features")
    fi
    cargo check "${args[@]}"
}

check default true "" "derive,runtime"
check minimal false "" ""
check derive false "derive" "derive"
check runtime false "runtime" "runtime"
check mcp false "mcp" "mcp"
check mcp-values false "mcp,chrono,rust_decimal" "mcp,chrono,rust_decimal"
