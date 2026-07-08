use es_fluent::unic_langid::LanguageIdentifier;
use gpui::BorrowAppContext as _;
use gpui_storybook::{Assets, Gallery};
use gpui_storybook_core::{
    language::Language,
    locale::{LocaleManager, LocaleStore},
};
use some_lib::i18n::Languages;

// bring the stories in scope for inventory
#[allow(unused_imports, clippy::single_component_path_imports)]
use some_lib_forms;

struct FormLocaleStore<L: Language> {
    inner: LocaleManager<L>,
}

impl<L: Language> FormLocaleStore<L> {
    fn new() -> Self {
        Self {
            inner: LocaleManager::new(),
        }
    }
}

impl<L: Language> LocaleStore for FormLocaleStore<L> {
    fn available_locales(
        &self,
        cx: &gpui::App,
    ) -> anyhow::Result<Vec<(String, LanguageIdentifier)>> {
        self.inner.available_locales(cx)
    }

    fn current_locale(&self, cx: &gpui::App) -> anyhow::Result<LanguageIdentifier> {
        self.inner.current_locale(cx)
    }

    fn set_current_locale(
        &self,
        locale: LanguageIdentifier,
        cx: &mut gpui::App,
    ) -> anyhow::Result<()> {
        self.inner.set_current_locale(locale.clone(), cx)?;
        gpui_es_fluent::change_locale(cx, locale.clone()).map_err(|err| {
            anyhow::anyhow!("failed to sync gpui-form locale to '{}': {err}", locale)
        })?;
        Ok(())
    }
}

fn main() {
    let app = gpui_platform::application().with_assets(Assets);
    let name_arg = std::env::args().nth(1);

    app.run(move |app_cx| {
        gpui_component::init(app_cx);
        gpui_es_fluent::replace_with_language(app_cx, Languages::default())
            .expect("failed to initialize form story i18n");
        gpui_storybook::init(app_cx, Languages::default());
        app_cx.set_global(Box::new(FormLocaleStore::<Languages>::new()) as Box<dyn LocaleStore>);
        app_cx
            .update_global::<Box<dyn LocaleStore>, _>(|locale_store, cx| {
                locale_store.set_current_locale(Languages::default().into(), cx)
            })
            .expect("default form story locale should be available");
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
