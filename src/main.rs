mod api;
mod cli;
mod config;
mod data;
mod display;
mod lang_colors;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, UserOpts};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let command = match (cli.command, cli.username) {
        (Some(cmd), _) => cmd,
        (None, Some(username)) => Command::User {
            username,
            opts: UserOpts::default_full(),
        },
        (None, None) => unreachable!("clap enforces arg_required_else_help"),
    };

    let token = config::resolve_token(cli.token).await;
    let client = api::client::GhClient::new(token.clone(), cli.verbose)?;

    match command {
        Command::User { username, opts } => {
            let profile = data::user::fetch_user_profile(&client, &username, &opts).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&profile)?);
            } else {
                display::user::render(&profile, &opts, cli.theme, cli.no_color);
            }
        }
        Command::Repo { repo, opts } => {
            let profile = data::repo::fetch_repo_profile(&client, &repo, &opts).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&profile)?);
            } else {
                display::repo::render(&profile, &opts, cli.theme, cli.no_color);
            }
        }
        Command::Org { orgname, opts } => {
            let profile = data::org::fetch_org_profile(&client, &orgname, &opts).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&profile)?);
            } else {
                display::org::render(&profile, &opts, cli.theme, cli.no_color);
            }
        }
    }

    Ok(())
}
