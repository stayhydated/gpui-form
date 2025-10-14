use gpui::Application;
use gpui_storybook::{Assets, Gallery};

// bring the stories in scope for inventory
#[allow(unused_imports)]
use some_lib_forms;

fn main() {
    let app = Application::new().with_assets(Assets);
    let name_arg = std::env::args().nth(1);

    app.run(move |app_cx| {
        gpui_component::init(app_cx);
        gpui_storybook::init(app_cx);
        gpui_storybook::change_locale("en").unwrap();
        app_cx.activate(true);

        gpui_storybook::create_new_window(
            &format!("{} - Stories", env!("CARGO_PKG_NAME")),
            move |window, cx| {
                let all_stories = gpui_storybook::generate_stories(window, cx);

                Gallery::view(all_stories, name_arg.as_deref(), window, cx)
            },
            app_cx,
        );
    });
}
