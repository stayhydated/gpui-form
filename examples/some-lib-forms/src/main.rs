use gpui_storybook::Assets;

fn main() {
    let app = gpui_kit::application().with_assets(Assets);
    app.run(some_lib_forms::launch_storybook);
}
