use stayhydated_xtask::trunk::{TrunkDemoBuildConfig, TrunkDemoPageConfig};

pub fn run() -> anyhow::Result<()> {
    let workspace_root = stayhydated_xtask::workspace_root_from_xtask_manifest()?;
    stayhydated_xtask::trunk::build(
        &TrunkDemoBuildConfig::builder()
            .workspace_root(workspace_root)
            .example_dir("examples/some-lib-forms")
            .output_dir("web/public/gpui-demo")
            .example_name("demo")
            .required_marker("gpui-form-some-lib-forms")
            .toolchain("nightly")
            .generated_page(
                TrunkDemoPageConfig::builder()
                    .title("some-lib-forms Storybook demo")
                    .demo_name("some-lib-forms Storybook")
                    .build(),
            )
            .build(),
    )
}
