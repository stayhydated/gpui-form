use es_fluent_manager_singleton as i18n_manager;

pub fn init() {
    i18n_manager::init();
}

pub fn change_locale(language: &str) -> Result<(), unic_langid::LanguageIdentifierError> {
    let lang_id: unic_langid::LanguageIdentifier = language.parse()?;
    i18n_manager::select_language(&lang_id);
    Ok(())
}
