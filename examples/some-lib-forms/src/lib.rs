use gpui_kit::Application;
use gpui_storybook::{ConsumerId, StorybookOptions, StorybookWindow};
use some_lib::i18n::{self, Languages};

pub mod forms;

const CONSUMER_ID: &str = "gpui-form-some-lib-forms";

fn storybook_options() -> Result<StorybookOptions<Languages>, gpui_storybook::ConsumerIdError> {
    let options = StorybookOptions::new(
        ConsumerId::new(CONSUMER_ID)?,
        Languages::default(),
        i18n::apply_locale,
    );

    #[cfg(target_family = "wasm")]
    let options = options.with_persistence(gpui_storybook::PersistenceMode::Disabled);

    Ok(options)
}

pub fn run_storybook(app: Application) {
    app.run(move |app_cx| {
        let options = match storybook_options() {
            Ok(options) => options,
            Err(error) => {
                eprintln!("invalid generated-form Storybook consumer id: {error}");
                app_cx.quit();
                return;
            },
        };
        let readiness = match gpui_storybook::init(app_cx, options) {
            Ok(readiness) => readiness,
            Err(error) => {
                eprintln!("failed to initialize generated-form Storybook: {error}");
                app_cx.quit();
                return;
            },
        };

        app_cx
            .spawn(async move |cx| {
                let ready = readiness.await;
                if !ready.diagnostics.is_empty() {
                    eprintln!(
                        "generated-form Storybook preferences initialized with diagnostics: {:?}",
                        ready.diagnostics
                    );
                }

                cx.update(|app_cx| {
                    app_cx.activate(true);
                    gpui_storybook::create_storybook_window(
                        &format!("{} - Stories", env!("CARGO_PKG_NAME")),
                        move |window, cx| {
                            let stories = gpui_storybook::generate_stories(window, cx);
                            assert!(
                                !stories.is_empty(),
                                "generated-form Storybook requires linked stories"
                            );
                            StorybookWindow::new(stories)
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
    use some_lib::structs::user::User;

    #[test]
    fn startup_contract_uses_a_stable_consumer_and_typed_adapter() {
        let consumer = ConsumerId::new(CONSUMER_ID).expect("checked consumer id");
        let options =
            StorybookOptions::new(consumer.clone(), Languages::default(), i18n::apply_locale);
        assert_eq!(options.consumer_id, consumer);
        assert_eq!(options.fallback_language, Languages::default());
    }

    #[test]
    fn binary_links_expected_story_registrations() {
        let mut story_keys =
            gpui_storybook::__inventory::iter::<gpui_storybook::__registry::StoryEntry>()
                .filter(|entry| entry.crate_name == env!("CARGO_PKG_NAME"))
                .map(|entry| entry.key.as_str())
                .collect::<Vec<_>>();
        story_keys.sort_unstable();

        assert_eq!(
            story_keys,
            [
                "some-lib-forms-EmptyForm",
                "some-lib-forms-ItemForm",
                "some-lib-forms-LocationFormForm",
                "some-lib-forms-UserForm",
            ]
        );
    }

    #[gpui_kit::test]
    async fn startup_applies_preferences_before_the_first_window(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let readiness = cx.update(|cx| {
            let consumer =
                ConsumerId::new("gpui-form-some-lib-forms-test").expect("checked test consumer id");
            gpui_storybook::init(
                cx,
                StorybookOptions::new(consumer, Languages::default(), i18n::apply_locale)
                    .with_persistence(gpui_storybook::PersistenceMode::Disabled)
                    .with_overrides(gpui_storybook::PreferenceOverrides {
                        language: Some(Languages::FrFr),
                        ..Default::default()
                    }),
            )
            .expect("generated-form Storybook should initialize")
        });
        let ready = readiness.await;
        assert_eq!(
            ready.persistence_status,
            gpui_storybook::PersistenceStatus::Ready
        );
        assert!(ready.diagnostics.is_empty());

        let (language, source, user_label) = cx.update(|cx| {
            let state = gpui_storybook::try_preference_state(cx)
                .expect("preference state should be installed after initialization");
            (
                state.resolved.language.language.to_string(),
                state.resolved.language.source,
                gpui_es_fluent::localize_label::<User>(cx),
            )
        });
        assert_eq!(language, "fr-FR");
        assert_eq!(source, gpui_storybook::LanguageSource::Override);
        assert_eq!(user_label, "Utilisateur");
    }
}
