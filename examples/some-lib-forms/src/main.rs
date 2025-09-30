use gpui::Application;
use gpui_storybook::{Assets, Gallery};
use some_lib::i18n;
use some_lib_forms::forms;

fn main() {
    let app = Application::new().with_assets(Assets);
    let name_arg = std::env::args().nth(1);

    app.run(move |app_cx| {
        i18n::init();
        i18n::change_locale("en").unwrap();
        gpui_component::init(app_cx);
        gpui_storybook::init(app_cx);
        forms::init(app_cx);
        app_cx.activate(true);

        gpui_storybook::create_new_window(
            &format!("{} - Stories", env!("CARGO_PKG_NAME")),
            move |window, cx| {
                let all_stories = forms::generate_stories(window, cx);

                Gallery::view(all_stories, name_arg.as_deref(), window, cx)
            },
            app_cx,
        );
    });
}
