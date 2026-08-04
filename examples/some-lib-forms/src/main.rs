use gpui_storybook::Assets;

fn main() {
    let app = gpui_platform::application().with_assets(Assets);
    let selected_story = std::env::args().nth(1);
    some_lib_forms::run_storybook(app, selected_story);
}
