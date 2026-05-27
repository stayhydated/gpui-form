es_fluent_manager_embedded::define_i18n_module!();

pub use gpui_es_fluent::{
    EmbeddedI18n, EmbeddedInitError, I18n, LocalizationError, change_locale, fallback_label,
    localize_label, localize_message, replace_with_language as init,
};
