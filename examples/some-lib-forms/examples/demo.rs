#[cfg(target_family = "wasm")]
use gpui::Application;
use gpui_storybook::Assets;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_family = "wasm"))]
fn main() {
    some_lib_forms::run_storybook(gpui_platform::application().with_assets(Assets));
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    gpui_platform::web_init();
    let app = keep_web_application_alive(gpui_platform::single_threaded_web()).with_assets(Assets);
    some_lib_forms::run_storybook(app);
    Ok(())
}

#[cfg(target_family = "wasm")]
fn main() {
    let _ = run();
}

#[cfg(target_family = "wasm")]
fn keep_web_application_alive(app: Application) -> Application {
    struct WasmApplication(std::rc::Rc<gpui::AppCell>);

    // GPUI keeps browser callbacks after `Application::run` returns.
    unsafe {
        let wasm_app = std::mem::transmute::<Application, WasmApplication>(app);
        std::mem::forget(wasm_app.0.clone());
        std::mem::transmute::<WasmApplication, Application>(wasm_app)
    }
}
