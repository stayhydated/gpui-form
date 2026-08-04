use dioxus::prelude::*;
use stayhydated_dioxus::{Project, ProjectSite, StayhydatedEmbeddedDemoProjectApp};

const PROJECT: Project = Project::new(
    "gpui-form",
    "Derive typed, component-backed GPUI forms from application models.",
)
.with_skill_command("npx skills add stayhydated/gpui-form");
const SITE_URL: &str = "https://stayhydated.github.io/gpui-form/";
const RUSTDOC_URL: &str = "https://docs.rs/gpui-form/";
const SOURCE_URL: &str = "https://github.com/stayhydated/gpui-form";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn site() -> ProjectSite {
    ProjectSite::builder()
        .project(PROJECT)
        .site_url(SITE_URL)
        .rustdoc_url(RUSTDOC_URL)
        .source_url(SOURCE_URL)
        .version(VERSION)
        .demo_path("gpui-demo")
        .build()
}

#[component]
pub fn App() -> Element {
    rsx! { StayhydatedEmbeddedDemoProjectApp { site: site() } }
}

pub fn route_manifest() -> stayhydated_site::SiteRouteManifest {
    site().embedded_demo_route_manifest()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_demo_site_tracks_the_frame_and_static_gpui_demo() {
        let site = site();

        assert_eq!(site.demo_path(), Some("gpui-demo"));
        assert_eq!(site.rustdoc_url(), RUSTDOC_URL);
        assert_eq!(site.source_url(), SOURCE_URL);
        assert_eq!(
            site.project().skill_command(),
            Some("npx skills add stayhydated/gpui-form")
        );
        assert_eq!(
            route_manifest()
                .application_paths()
                .iter()
                .map(|path| path.as_str())
                .collect::<Vec<_>>(),
            ["/", "/demo/"]
        );
        assert!(
            route_manifest()
                .static_paths()
                .iter()
                .any(|path| path.as_str() == "/gpui-demo/")
        );
    }
}
