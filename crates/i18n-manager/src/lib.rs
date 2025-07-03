use i18n_embed::unic_langid::LanguageIdentifier;
use std::sync::{Arc, RwLock};

pub trait I18nModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn init(&self, requested_languages: &[LanguageIdentifier]) -> anyhow::Result<()>;
    fn change_locale(&self, language: &str) -> anyhow::Result<()>;
}

inventory::collect!(&'static dyn I18nModule);

pub struct I18nManager {
    current_language: Arc<RwLock<Option<String>>>,
    initialized: Arc<RwLock<bool>>,
}

impl I18nManager {
    pub fn new() -> Self {
        Self {
            current_language: Arc::new(RwLock::new(None)),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    pub fn init_all(&self, requested_languages: &[LanguageIdentifier]) -> anyhow::Result<()> {
        let mut initialized = self.initialized.write().unwrap();
        if *initialized {
            return Ok(());
        }

        for module in inventory::iter::<&'static dyn I18nModule> {
            module.init(requested_languages)?;
        }

        if let Some(lang) = requested_languages.first() {
            *self.current_language.write().unwrap() = Some(lang.to_string());
        }

        *initialized = true;
        Ok(())
    }

    pub fn change_locale_all(&self, language: &str) -> anyhow::Result<()> {
        let mut errors = Vec::new();

        for module in inventory::iter::<&'static dyn I18nModule> {
            if let Err(e) = module.change_locale(language) {
                errors.push(format!("Module {}: {}", module.name(), e));
            }
        }

        if errors.is_empty() {
            *self.current_language.write().unwrap() = Some(language.to_string());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Errors in modules: {}", errors.join(", ")))
        }
    }

    pub fn current_language(&self) -> Option<String> {
        self.current_language.read().unwrap().clone()
    }
}

use std::sync::LazyLock;
pub static I18N_MANAGER: LazyLock<I18nManager> = LazyLock::new(I18nManager::new);
