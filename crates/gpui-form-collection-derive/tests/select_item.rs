use std::{fs, path::PathBuf, process::Command};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("collection derive crate lives under crates/gpui-form-collection-derive")
        .to_path_buf()
}

#[test]
fn select_item_accepts_no_display_and_generic_enums() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/select_item_no_display.rs");
    tests.pass("tests/ui/select_item_generic.rs");
}

#[test]
fn select_item_supports_renamed_gpui_dependencies() {
    let workspace = workspace_root();
    let crate_dir = workspace.join("target/select-item-renamed-dependency-check");
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).expect("create renamed dependency test crate");

    fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "select-item-renamed-dependency-check"
version = "0.0.0"
edition = "2024"

[workspace]

[dependencies]
renamed-gpui = {{ package = "gpui", git = "https://github.com/zed-industries/zed", rev = "832c17e8192e2e1d472f0751e7cef2af84ded622" }}
renamed-gpui-component = {{ package = "gpui-component", git = "https://github.com/longbridge/gpui-component", rev = "e64411fcbcf4eb586334a4d543257218e009114b" }}
gpui-form-collection-derive = {{ path = "{derive}" }}

[replace]
"https://github.com/zed-industries/zed#gpui@0.2.2" = {{ git = "https://github.com/zed-industries/zed", rev = "832c17e8192e2e1d472f0751e7cef2af84ded622" }}
"#,
            derive = workspace
                .join("crates/gpui-form-collection-derive")
                .display(),
        ),
    )
    .expect("write renamed dependency test manifest");

    fs::write(
        src_dir.join("lib.rs"),
        r#"
use renamed_gpui_component::select::SelectItem as _;

#[derive(Clone, PartialEq, gpui_form_collection_derive::SelectItem)]
pub enum Status {
    Ready,
}

pub fn title() -> renamed_gpui::SharedString {
    Status::Ready.title()
}
"#,
    )
    .expect("write renamed dependency test source");

    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .current_dir(&workspace)
        .output()
        .expect("run cargo check for renamed dependency test crate");

    assert!(
        output.status.success(),
        "renamed dependency test crate failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
