use std::{fs, path::PathBuf, process::Command};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("derive crate lives under crates/gpui-form-derive")
        .to_path_buf()
}

fn component_shape_root(workspace: &std::path::Path) -> PathBuf {
    workspace
        .parent()
        .expect("gpui-form has a parent directory")
        .join("component-shape/crates")
}

#[test]
fn gpui_form_derive_uses_facade_runtime_reexport() {
    let workspace = workspace_root();
    let component_shape_root = component_shape_root(&workspace);
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
component-shape = {{ path = "{component_shape}" }}
component-shape-gpui = {{ path = "{component_shape_gpui}" }}
gpui-form = {{ path = "{gpui_form}", default-features = false, features = ["derive"] }}
gpui-form-runtime = {{ path = "{runtime}" }}
"#,
            component_shape = component_shape_root.join("component-shape").display(),
            component_shape_gpui = component_shape_root.join("component-shape-gpui").display(),
            gpui_form = workspace.join("crates/gpui-form").display(),
            runtime = workspace.join("crates/gpui-form-runtime").display(),
        ),
    )
    .expect("write renamed dependency test manifest");

    fs::write(
        src_dir.join("lib.rs"),
        r#"
struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape_gpui::component_shape! {
    struct RenamedRuntimeShape {
        type State = State;
        value = String;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for RenamedRuntimeShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::RequiredValueStorage;
}

#[derive(gpui_form::GpuiForm)]
pub struct Demo {
    #[gpui_form(component(RenamedRuntimeShape))]
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
fn component_shape_contracts_support_renamed_dependencies() {
    let workspace = workspace_root();
    let component_shape_root = component_shape_root(&workspace);
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
renamed-component-shape = {{ package = "component-shape", path = "{component_shape}" }}
renamed-component-shape-gpui = {{ package = "component-shape-gpui", path = "{component_shape_gpui}" }}
renamed-gpui-form-runtime = {{ package = "gpui-form-runtime", path = "{runtime}" }}
"#,
            component_shape = component_shape_root.join("component-shape").display(),
            component_shape_gpui = component_shape_root.join("component-shape-gpui").display(),
            runtime = workspace.join("crates/gpui-form-runtime").display(),
        ),
    )
    .expect("write renamed component macro test manifest");

    fs::write(
        src_dir.join("lib.rs"),
        r#"
use renamed_component_shape_gpui::component_shape;
use renamed_gpui_form_runtime::shape::{
    DeclaredGpuiComponentShape, DirectValueStorage, GpuiComponentShape,
    GpuiComponentStateValueBinding, GpuiComponentValueBinding, GpuiFormComponentShapePolicy,
    RequiredValueStorage, ValueChange,
};

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

impl DerivedComponent {
    fn new(_entity: &renamed_gpui::Entity<DerivedState>) -> renamed_gpui::Div {
        renamed_gpui::div()
    }
}

impl renamed_gpui::EventEmitter<DerivedEvent> for DerivedState {}

impl GpuiComponentStateValueBinding<String> for DerivedState {
    type Event = DerivedEvent;

    fn value_change(_state: &Self, _event: &Self::Event) -> ValueChange<String> {
        ValueChange::Unchanged
    }
}

#[derive(renamed_component_shape_gpui::GpuiComponentShape)]
#[gpui_component_shape(
    state = DerivedState,
    value = String,
    value_binding,
    field_suffix = "derived"
)]
pub struct DerivedComponent;

impl GpuiFormComponentShapePolicy for DerivedComponent {
    type ValueStoragePolicy = RequiredValueStorage;
}

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

impl MacroComponent {
    fn new(_entity: &renamed_gpui::Entity<MacroState>) -> renamed_gpui::Div {
        renamed_gpui::div()
    }
}

impl renamed_gpui::EventEmitter<MacroEvent> for MacroState {}

component_shape! {
    pub struct MacroShape {
        type State = MacroState;
        component = MacroComponent;
        value = String;
        field_suffix = "macro";
        value_binding;

        impl GpuiComponentValueBinding<String> for MacroShape {
            type Event = MacroEvent;

            fn value_change(
                _state: &Self::State,
                _event: &Self::Event,
            ) -> ValueChange<String> {
                ValueChange::Unchanged
            }
        }
    }
}

impl GpuiFormComponentShapePolicy for MacroShape {
    type ValueStoragePolicy = DirectValueStorage;
}

pub fn assert_shapes() {
    fn assert_shape<Shape: GpuiComponentShape + DeclaredGpuiComponentShape>() {}
    fn assert_binding<Shape, Event>()
    where
        Shape: GpuiComponentValueBinding<String, Event = Event>,
        <Shape as GpuiComponentShape>::State: renamed_gpui::EventEmitter<Event>,
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
