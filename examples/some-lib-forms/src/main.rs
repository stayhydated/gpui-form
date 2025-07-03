use gpui::Application;
use some_lib_forms::forms;
use story_container::{Assets, Gallery};

fn main() {
    let app = Application::new().with_assets(Assets);
    let name_arg = std::env::args().nth(1);

    app.run(move |app_cx| {
        gpui_component::init(app_cx);
        story_container::story::init(app_cx);
        forms::init(app_cx);
        app_cx.activate(true);

        story_container::story::create_new_window(
            &format!("{} - Stories", env!("CARGO_PKG_NAME")),
            move |window, cx| {
                let all_stories = forms::generate_stories(window, cx);

                Gallery::view(all_stories, name_arg.as_deref(), window, cx)
            },
            app_cx,
        );
    });
}
