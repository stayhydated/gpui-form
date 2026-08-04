use stayhydated_xtask::preview::StaticSitePreviewConfig;

pub fn run() -> anyhow::Result<()> {
    let workspace_root = stayhydated_xtask::workspace_root_from_xtask_manifest()?;
    stayhydated_xtask::preview::serve(
        &StaticSitePreviewConfig::builder()
            .workspace_root(&workspace_root)
            .dist_dir("web/dist")
            .base_path("gpui-form")
            .build_hint("Run `just web-build` first.")
            .build(),
    )
}
