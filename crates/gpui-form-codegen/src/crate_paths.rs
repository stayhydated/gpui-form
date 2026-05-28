use proc_macro_crate::{FoundCrate, crate_name};
use syn::{Path, parse_quote};

#[derive(Clone, Debug)]
pub struct CratePaths {
    pub gpui: Path,
    pub gpui_form: Path,
    pub gpui_form_runtime: Path,
}

impl CratePaths {
    pub fn resolve() -> Self {
        Self {
            gpui: resolve_crate_path("gpui", "::gpui"),
            gpui_form: resolve_crate_path("gpui-form", "::gpui_form"),
            gpui_form_runtime: resolve_crate_path("gpui-form-runtime", "::gpui_form_runtime"),
        }
    }

    pub fn gpui_form_facade_runtime(&self) -> Path {
        let mut path = self.gpui_form.clone();
        path.segments.push(parse_quote!(runtime));
        path
    }
}

fn resolve_crate_path(package_name: &str, fallback: &str) -> Path {
    let path = match crate_name(package_name) {
        Ok(FoundCrate::Itself) => "crate".to_string(),
        Ok(FoundCrate::Name(name)) => format!("::{name}"),
        Err(_) => fallback.to_string(),
    };

    syn::parse_str(&path).expect("crate path resolver produced a valid Rust path")
}
