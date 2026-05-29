use std::{fs, path::PathBuf, process::Command};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("derive crate lives under crates/gpui-form-derive")
        .to_path_buf()
}

#[test]
fn gpui_form_derive_uses_facade_runtime_reexport() {
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
"#,
            gpui_form = workspace.join("crates/gpui-form").display(),
        ),
    )
    .expect("write renamed dependency test manifest");

    fs::write(
        src_dir.join("lib.rs"),
        r#"
use gpui_form::runtime::shape::{
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

#[test]
fn component_shape_macros_support_renamed_gpui_and_runtime_dependencies() {
    let workspace = workspace_root();
    let crate_dir = workspace.join("target/renamed-component-macro-check");
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).expect("create renamed component macro test crate");

    fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "renamed-component-macro-check"
version = "0.0.0"
edition = "2024"

[workspace]

[dependencies]
renamed-gpui = {{ package = "gpui", git = "https://github.com/zed-industries/zed", rev = "832c17e8192e2e1d472f0751e7cef2af84ded622" }}
gpui-form-derive = {{ path = "{derive}" }}
renamed-gpui-form-runtime = {{ package = "gpui-form-runtime", path = "{runtime}" }}
"#,
            derive = workspace.join("crates/gpui-form-derive").display(),
            runtime = workspace.join("crates/gpui-form-runtime").display(),
        ),
    )
    .expect("write renamed component macro test manifest");

    fs::write(
        src_dir.join("lib.rs"),
        r#"
use renamed_gpui_form_runtime::shape::{ComponentShape, ComponentValueBinding, FormValueChange};

pub struct DerivedState;

impl DerivedState {
    fn new(
        _window: &mut renamed_gpui::Window,
        _cx: &mut renamed_gpui::Context<'_, Self>,
    ) -> Self {
        Self
    }
}

pub struct DerivedEvent;

impl renamed_gpui::EventEmitter<DerivedEvent> for DerivedState {}

#[gpui_form_derive::component_value_binding]
impl ComponentValueBinding<String> for DerivedState {
    type Event = DerivedEvent;

    fn form_value_change(_state: &Self, _event: &Self::Event) -> FormValueChange<String> {
        FormValueChange::Unchanged
    }
}

#[derive(gpui_form_derive::ComponentShape)]
#[gpui_form_shape(state = DerivedState, value_binding, field_suffix = "derived")]
pub struct DerivedComponent;

pub struct MacroState;

impl MacroState {
    fn new(
        _window: &mut renamed_gpui::Window,
        _cx: &mut renamed_gpui::Context<'_, Self>,
    ) -> Self {
        Self
    }
}

pub struct MacroComponent;
pub struct MacroEvent;

impl renamed_gpui::EventEmitter<MacroEvent> for MacroState {}

gpui_form_derive::component_shape! {
    pub struct MacroShape {
        type State = MacroState;
        component = MacroComponent;
        requires_value = false;
        field_suffix = "macro";
        value_binding;

        impl ComponentValueBinding<String> for MacroShape {
            type Event = MacroEvent;

            fn form_value_change(
                _state: &Self::State,
                _event: &Self::Event,
            ) -> FormValueChange<String> {
                FormValueChange::Unchanged
            }
        }
    }
}

pub fn assert_shapes() {
    fn assert_shape<Shape: ComponentShape>() {}
    fn assert_binding<Shape, Event>()
    where
        Shape: ComponentValueBinding<String, Event = Event>,
        <Shape as ComponentShape>::State: renamed_gpui::EventEmitter<Event>,
        Event: 'static,
    {
    }

    assert_shape::<DerivedComponent>();
    assert_shape::<MacroShape>();
    assert_binding::<DerivedComponent, DerivedEvent>();
    assert_binding::<MacroShape, MacroEvent>();
}
"#,
    )
    .expect("write renamed component macro test source");

    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .current_dir(&workspace)
        .output()
        .expect("run cargo check for renamed component macro test crate");

    assert!(
        output.status.success(),
        "renamed component macro test crate failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
