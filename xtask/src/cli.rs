use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    about = "Workspace documentation and web tasks.",
    disable_help_subcommand = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build a generated documentation or web artifact
    Build {
        #[command(subcommand)]
        target: BuildCommand,
    },
    /// Preview a generated documentation or web artifact
    Preview {
        #[command(subcommand)]
        target: PreviewCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum BuildCommand {
    /// Build mdBook documentation to web/public/book
    Book,
    /// Build the Trunk-hosted GPUI demo to web/public/gpui-demo
    GpuiDemo,
    /// Build llms.txt and per-chapter Markdown from mdBook sources
    LlmsTxt,
    /// Build the Dioxus site into web/dist for GitHub Pages
    Web,
}

#[derive(Debug, Subcommand)]
pub enum PreviewCommand {
    /// Preview the generated static site with its GitHub Pages base path
    Web,
}
