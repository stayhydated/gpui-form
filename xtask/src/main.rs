mod cli;
mod commands;

use clap::Parser as _;

use cli::{BuildCommand, Cli, Command, PreviewCommand};

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Build { target } => match target {
            BuildCommand::Book => commands::build_book::run(),
            BuildCommand::GpuiDemo => commands::build_gpui_demo::run(),
            BuildCommand::LlmsTxt => commands::build_llms_txt::run(),
            BuildCommand::Web => commands::build_web::run(),
        },
        Command::Preview { target } => match target {
            PreviewCommand::Web => commands::preview_web::run(),
        },
    }
}
