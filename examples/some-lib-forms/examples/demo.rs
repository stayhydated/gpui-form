#[cfg(target_family = "wasm")]
use std::cell::RefCell;

use gpui_storybook::Assets;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_family = "wasm"))]
fn main() {
    gpui_kit::application()
        .with_assets(Assets)
        .run(some_lib_forms::launch_storybook);
}

#[cfg(target_family = "wasm")]
thread_local! {
    static APPLICATION: RefCell<Option<gpui_kit::ApplicationHandle>> = const { RefCell::new(None) };
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    gpui_kit::platform::web_init();
    let app = gpui_kit::platform::single_threaded_web().with_assets(Assets);
    APPLICATION.with(|application| {
        *application.borrow_mut() = Some(app.run_embedded(some_lib_forms::launch_storybook));
    });
    Ok(())
}

#[cfg(target_family = "wasm")]
fn main() {}
