use anyhow::Context as _;
use poise::{Command, command};

use super::{Context, Data, Error, Result};

pub(super) fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![shadowbox()]
}

#[command(slash_command, subcommands("challenge"))]
pub(super) async fn shadowbox(ctx: Context<'_>) -> Result {
    ctx.say("Invalid Command!")
        .await
        .context("Failed to send message")?;
    Ok(())
}

#[command(slash_command)]
pub(super) async fn challenge(ctx: Context<'_>) -> Result {
    ctx.say("Invalid Command!")
        .await
        .context("Failed to send message")?;
    Ok(())
}
