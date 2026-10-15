#!/usr/bin/env bash
set -euo pipefail
repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"
consumer_dir=$(mktemp -d)
trap 'rm -rf "$consumer_dir"' EXIT
mkdir "$consumer_dir/src"
cp .github/reconstruction-lints.rs "$consumer_dir/src/main.rs"
cat > "$consumer_dir/Cargo.toml" <<EOF
[package]
name = "gpui-form-reconstruction-lints"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
gpui-form = { path = "$repo_root/crates/gpui-form" }
EOF
cp Cargo.lock "$consumer_dir/Cargo.lock"
cargo metadata --manifest-path "$consumer_dir/Cargo.toml" --offline --format-version 1 > /dev/null
args=(--manifest-path "$consumer_dir/Cargo.toml" --target-dir "$repo_root/target/feature-consumers" --offline --locked)
for threshold in 3 7 12; do
    echo "Checking skipped-field reconstruction with Clippy threshold $threshold"
    printf 'too-many-arguments-threshold = %s\n' "$threshold" > "$consumer_dir/clippy.toml"
    printf '// Clippy threshold %s\n' "$threshold" >> "$consumer_dir/src/main.rs"
    CLIPPY_CONF_DIR="$consumer_dir" cargo clippy "${args[@]}" -- -D warnings -D clippy::too_many_arguments
done
# An allowed lint must not produce an unfulfilled expectation either.
CLIPPY_CONF_DIR="$consumer_dir" cargo clippy "${args[@]}" -- -D warnings -A clippy::too_many_arguments
# Prove that the generated allowance does not silence application-owned APIs.
printf 'too-many-arguments-threshold = 7\n' > "$consumer_dir/clippy.toml"
cat >> "$consumer_dir/src/main.rs" <<'RS'
pub fn application_owned(a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64, h: u64) -> u64 {
    a + b + c + d + e + f + g + h
}
RS
if CLIPPY_CONF_DIR="$consumer_dir" cargo clippy "${args[@]}" --message-format=json -- -D warnings -D clippy::too_many_arguments > "$consumer_dir/control.json" 2> "$consumer_dir/control.log"; then
    echo 'Expected the application-owned method to trigger too_many_arguments' >&2
    exit 1
fi
if ! grep -q '"code":"clippy::too_many_arguments"' "$consumer_dir/control.json"; then
    cat "$consumer_dir/control.json" "$consumer_dir/control.log" >&2
    exit 1
fi
cp .github/reconstruction-lints.rs "$consumer_dir/src/main.rs"
cargo run "${args[@]}"
