use i18n_embed::unic_langid::LanguageIdentifier;
use i18n_manager::I18N_MANAGER;

pub fn init_i18n() -> anyhow::Result<()> {
    let requested_languages = available_languages()
        .iter()
        .map(|lang| lang.parse().unwrap())
        .collect::<Vec<LanguageIdentifier>>();

    I18N_MANAGER.init_all(&requested_languages)
}

pub fn change_language(language: &str) -> anyhow::Result<()> {
    println!("Changing language to {}", language);
    I18N_MANAGER.change_locale_all(language)
}

pub fn current_language() -> Option<String> {
    I18N_MANAGER.current_language()
}

pub fn available_languages() -> Vec<&'static str> {
    vec!["en", "zh-CN", "fr"]
}
