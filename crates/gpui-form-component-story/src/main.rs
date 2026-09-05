use gpui_form_component_story::i18n::{self, Languages};
use gpui_storybook::{Assets, ConsumerId, StorybookOptions, StorybookWindow};

// Bring the library target into scope so story inventory registrations are linked.
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_form_component_story;

const CONSUMER_ID: &str = "gpui-form-component-story";

fn main() {
    let app = gpui_kit::application().with_assets(Assets);
    app.run(move |app_cx| {
        let consumer_id = match ConsumerId::new(CONSUMER_ID) {
            Ok(consumer_id) => consumer_id,
            Err(error) => {
                eprintln!("invalid component Storybook consumer id: {error}");
                app_cx.quit();
                return;
            },
        };
        let options = StorybookOptions::new(consumer_id, Languages::default(), i18n::apply_locale);
        let readiness = match gpui_storybook::init(app_cx, options) {
            Ok(readiness) => readiness,
            Err(error) => {
                eprintln!("failed to initialize component Storybook: {error}");
                app_cx.quit();
                return;
            },
        };

        app_cx
            .spawn(async move |cx| {
                let ready = readiness.await;
                if !ready.diagnostics.is_empty() {
                    eprintln!(
                        "component Storybook preferences initialized with diagnostics: {:?}",
                        ready.diagnostics
                    );
                }

                cx.update(|app_cx| {
                    app_cx.activate(true);
                    gpui_storybook::create_storybook_window(
                        &format!("{} - Stories", env!("CARGO_PKG_NAME")),
                        move |window, cx| {
                            let all_stories = gpui_storybook::generate_stories(window, cx);
                            StorybookWindow::new(all_stories)
                        },
                        app_cx,
                    );
                });
            })
            .detach();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit as gpui;

    #[test]
    fn startup_contract_uses_a_stable_consumer_and_typed_adapter() {
        let consumer = ConsumerId::new(CONSUMER_ID).expect("checked consumer id");
        let options =
            StorybookOptions::new(consumer.clone(), Languages::default(), i18n::apply_locale);
        assert_eq!(options.consumer_id, consumer);
        assert_eq!(options.fallback_language, Languages::default());
    }

    #[gpui_kit::test]
    async fn startup_applies_preferences_before_the_first_window(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let readiness = cx.update(|cx| {
            let consumer = ConsumerId::new("gpui-form-component-story-test")
                .expect("checked test consumer id");
            gpui_storybook::init(
                cx,
                StorybookOptions::new(consumer, Languages::default(), i18n::apply_locale)
                    .with_persistence(gpui_storybook::PersistenceMode::Disabled)
                    .with_overrides(gpui_storybook::PreferenceOverrides {
                        language: Some(Languages::FrFr),
                        ..Default::default()
                    }),
            )
            .expect("component Storybook should initialize")
        });
        let ready = readiness.await;
        assert_eq!(
            ready.persistence_status,
            gpui_storybook::PersistenceStatus::Ready
        );
        assert!(ready.diagnostics.is_empty());

        let (language, source) = cx.update(|cx| {
            let state = gpui_storybook::try_preference_state(cx)
                .expect("preference state should be installed after initialization");
            (
                state.resolved.language.language.to_string(),
                state.resolved.language.source,
            )
        });
        assert_eq!(language, "fr-FR");
        assert_eq!(source, gpui_storybook::LanguageSource::Override);
    }
}
