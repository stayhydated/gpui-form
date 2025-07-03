use i18n_embed::{
    DefaultLocalizer, LanguageLoader as _, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
    unic_langid::LanguageIdentifier,
};
use i18n_manager::I18nModule as I18nModuleTrait;
use rust_embed::RustEmbed;
use std::sync::LazyLock;

#[derive(RustEmbed)]
#[folder = "../i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    #[cfg(test)]
    loader.set_use_isolating(false);

    loader
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

#[must_use]
fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

struct I18nModule;

impl I18nModuleTrait for I18nModule {
    fn name(&self) -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn init(&self, requested_languages: &[LanguageIdentifier]) -> anyhow::Result<()> {
        localizer().select(requested_languages)?;
        Ok(())
    }

    fn change_locale(&self, language: &str) -> anyhow::Result<()> {
        let lang_id: LanguageIdentifier = language.parse()?;

        let requested_languages = vec![lang_id];

        localizer().select(&requested_languages)?;

        Ok(())
    }
}

inventory::submit! {
    &I18nModule as &'static dyn I18nModuleTrait
}
