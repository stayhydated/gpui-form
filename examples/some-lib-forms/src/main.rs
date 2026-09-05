use gpui_storybook::Assets;

fn main() {
    let app = gpui_kit::application().with_assets(Assets);
    some_lib_forms::run_storybook(app);
}
