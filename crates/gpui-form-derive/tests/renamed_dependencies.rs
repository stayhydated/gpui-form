use std::{fs, path::PathBuf, process::Command};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("derive crate lives under crates/gpui-form-derive")
        .to_path_buf()
}

#[test]
fn gpui_form_derive_supports_renamed_runtime_dependency() {
    let workspace = workspace_root();
    let crate_dir = workspace.join("target/renamed-dependency-check");
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).expect("create renamed dependency test crate");

    fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "renamed-dependency-check"
version = "0.0.0"
edition = "2024"

[workspace]

[dependencies]
gpui = {{ git = "https://github.com/zed-industries/zed", rev = "832c17e8192e2e1d472f0751e7cef2af84ded622" }}
gpui-form = {{ path = "{gpui_form}", default-features = false, features = ["derive"] }}
renamed-gpui-form-runtime = {{ package = "gpui-form-runtime", path = "{runtime}" }}
"#,
            gpui_form = workspace.join("crates/gpui-form").display(),
            runtime = workspace.join("crates/gpui-form-runtime").display(),
        ),
    )
    .expect("write renamed dependency test manifest");

    fs::write(
        src_dir.join("lib.rs"),
        r#"
use renamed_gpui_form_runtime::shape::{
    ComponentShape, NoComponentValueBinding, RequireValue,
};

struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

struct RenamedRuntimeShape;

impl ComponentShape for RenamedRuntimeShape {
    type State = State;
    type RequiredValuePolicy = RequireValue;
    type ValueBindingPolicy = NoComponentValueBinding;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

#[derive(gpui_form::GpuiForm)]
pub struct Demo {
    #[gpui_form(RenamedRuntimeShape)]
    pub name: String,
}

pub fn holder() -> DemoFormValueHolder {
    DemoFormValueHolder::default()
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
