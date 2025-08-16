use std::{time::Duration, vec};

use anyhow::Context as _;
use poise::{Command, CreateReply, command};
use serenity::all::{
    ComponentInteractionDataKind, CreateActionRow, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};

use crate::{db, localize_message, settings::get_context_settings};

use super::{Context, Data, Error, Result};

pub(super) fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![settings()]
}

#[command(
    slash_command,
    name_localized("en-US", "settings"),
    name_localized("es-419", "ajustes"),
    subcommands("user")
)]
pub(super) async fn settings(_ctx: Context<'_>) -> Result {
    unreachable!();
}

#[command(
    slash_command,
    name_localized("en-US", "user"),
    name_localized("es-419", "usuario"),
    description_localized("en-US", "Open the settings menu for yourself"),
    description_localized("es-419", "Abre el menú de ajustes para ti mismo")
)]
pub(super) async fn user(ctx: Context<'_>) -> Result {
    // Get the context settings
    let context_settings = get_context_settings(&ctx, &ctx.data().db)
        .await
        .context("Failed to get context settings")?;

    // Create the reply
    let reply = {
        // Create the components
        let components = vec![
            CreateActionRow::SelectMenu(CreateSelectMenu::new(
                "menu",
                CreateSelectMenuKind::String {
                    options: vec![CreateSelectMenuOption::new(
                        localize_message!("settings.language.label", &context_settings.language,)
                            .await
                            .context("Failed to localize message")?,
                        "language",
                    )],
                },
            )),
            CreateActionRow::Buttons(vec![
                CreateButton::new("viewbutton").label(
                    localize_message!(
                        "command.settings.user.button.view.label",
                        &context_settings.language
                    )
                    .await
                    .context("Failed to localize message")?,
                ),
                CreateButton::new("editbutton").label(
                    localize_message!(
                        "command.settings.user.button.edit.label",
                        &context_settings.language
                    )
                    .await
                    .context("Failed to localize message")?,
                ),
            ]),
        ];

        // Send the message
        ctx.send(
            CreateReply::default()
                .content(
                    localize_message!(
                        "command.settings.user.response.initial",
                        &context_settings.language
                    )
                    .await
                    .context("Failed to localize message")?,
                )
                .components(components)
                .ephemeral(true),
        )
        .await
        .context("Failed to send message")?
    };

    // Get interactions
    while let Some(interaction) = reply
        .message()
        .await
        .context("Failed to get message from reply")?
        .await_component_interaction(ctx)
        .timeout(Duration::from_secs(120))
        .await
    {
        // Respond to the interaction on the component
        match interaction.data.custom_id.as_str() {
            "menu" => {
                if let ComponentInteractionDataKind::StringSelect { values } =
                    &interaction.data.kind
                {
                    // Store the user's selection
                    let value = &values[0];
                    ctx.data().menu_selections.insert(
                        (interaction.message.id, interaction.user.id),
                        value.to_owned(),
                    );
                } else {
                    unreachable!("Invalid select kind");
                }
                interaction
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await
                    .context("Failed to acknowledge interaction")?;
            }
            "viewbutton" => {
                // Defer the interaction
                interaction
                    .defer_ephemeral(ctx)
                    .await
                    .context("Failed to defer interaction")?;

                // Get the selection
                let selection = ctx
                    .data()
                    .menu_selections
                    .get(&(interaction.message.id, interaction.user.id));

                if let Some(selection) = selection {
                    // Get the name of the setting
                    let setting_name = match selection.as_str() {
                        "language" => {
                            localize_message!("settings.language.label", &context_settings.language)
                                .await
                                .context("Failed to localize message")?
                        }
                        _ => unreachable!("Invalid selection"),
                    };

                    // Get the value of the setting
                    let setting_value = match selection.as_str() {
                        "language" => context_settings.language.to_string(),
                        _ => unreachable!("Invalid selection"),
                    };

                    // Create the followup response
                    interaction
                        .create_followup(
                            ctx,
                            CreateInteractionResponseFollowup::default()
                                .content(
                                    localize_message!(
                                        "command.settings.user.response.view",
                                        &context_settings.language,
                                        setting_name,
                                        setting_value
                                    )
                                    .await
                                    .context("Failed to localize message")?,
                                )
                                .ephemeral(true),
                        )
                        .await
                        .context("Failed to send followup message")?;
                } else {
                    // Tell the user no selection has been made
                    interaction
                        .create_followup(
                            ctx,
                            CreateInteractionResponseFollowup::default()
                                .content(
                                    localize_message!(
                                        "command.settings.user.response.error.noselection",
                                        &context_settings.language
                                    )
                                    .await
                                    .context("Failed to localize message")?,
                                )
                                .ephemeral(true),
                        )
                        .await
                        .context("Failed to send followup message")?;
                }
            }
            "editbutton" => {
                // Defer the interaction
                interaction
                    .defer_ephemeral(ctx)
                    .await
                    .context("Failed to defer interaction")?;

                // Get the selection
                let selection = ctx
                    .data()
                    .menu_selections
                    .get(&(interaction.message.id, interaction.user.id));

                if let Some(selection) = selection {
                    // Get the name of the setting
                    let setting_name = match selection.as_str() {
                        "language" => {
                            localize_message!("settings.language.label", &context_settings.language)
                                .await
                                .context("Failed to localize message")?
                        }
                        _ => unreachable!("Invalid selection"),
                    };

                    // Get the valid values for the setting, along with their labels
                    let valid_values = match selection.as_str() {
                        "language" => {
                            vec![
                                (
                                    "en",
                                    localize_message!(
                                        "settings.language.names.en",
                                        &context_settings.language
                                    )
                                    .await
                                    .context("Failed to localize message")?,
                                ),
                                (
                                    "es",
                                    localize_message!(
                                        "settings.language.names.es",
                                        &context_settings.language
                                    )
                                    .await
                                    .context("Failed to localize message")?,
                                ),
                            ]
                        }
                        _ => unreachable!("Invalid selection"),
                    };

                    // Create the followup response components
                    let components = vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
                        "menu",
                        CreateSelectMenuKind::String {
                            options: valid_values
                                .iter()
                                .map(|value| {
                                    CreateSelectMenuOption::new(
                                        value.1.to_owned(),
                                        value.0.to_owned(),
                                    )
                                })
                                .collect(),
                        },
                    ))];

                    // Create the followup response
                    let reply = interaction
                        .create_followup(
                            ctx,
                            CreateInteractionResponseFollowup::default()
                                .content(
                                    localize_message!(
                                        "command.settings.user.response.edit.initial",
                                        &context_settings.language,
                                        &setting_name
                                    )
                                    .await
                                    .context("Failed to localize message")?,
                                )
                                .components(components)
                                .ephemeral(true),
                        )
                        .await
                        .context("Failed to send followup message")?;

                    // Get interactions
                    while let Some(inner_interaction) = reply
                        .await_component_interaction(ctx)
                        .timeout(Duration::from_secs(120))
                        .await
                    {
                        // Respond to the interaction on the component
                        match inner_interaction.data.custom_id.as_str() {
                            "menu" => {
                                // Defer the interaction
                                inner_interaction
                                    .defer_ephemeral(ctx)
                                    .await
                                    .context("Failed to defer interaction")?;

                                // Get the selection
                                if let ComponentInteractionDataKind::StringSelect { values } =
                                    &inner_interaction.data.kind
                                {
                                    // Get the user's selection
                                    let inner_selection = &values[0];

                                    // Update the setting
                                    match selection.as_str() {
                                        "language" => match inner_selection.as_str() {
                                            "en" => {
                                                db::user_settings::update_entry(
                                                    &ctx.data().db,
                                                    inner_interaction.user.id,
                                                    db::user_settings::Column::Language,
                                                    db::user_settings::Language::English,
                                                )
                                                .await
                                                .context("Failed to update user settings")?;
                                            }
                                            "es" => {
                                                db::user_settings::update_entry(
                                                    &ctx.data().db,
                                                    inner_interaction.user.id,
                                                    db::user_settings::Column::Language,
                                                    db::user_settings::Language::Spanish,
                                                )
                                                .await
                                                .context("Failed to update user settings")?;
                                            }
                                            _ => unreachable!("Invalid selection"),
                                        },
                                        _ => unreachable!("Invalid selection"),
                                    }

                                    // Acknowledge the interaction
                                    inner_interaction
                                        .create_followup(
                                            ctx,
                                            CreateInteractionResponseFollowup::default()
                                                .content(
                                                    localize_message!(
                                            "command.settings.user.response.edit.success",
                                            &context_settings.language,
                                            &setting_name
                                        )
                                        .await
                                        .context("Failed to localize message")?,
                                                )
                                                .ephemeral(true),
                                        )
                                        .await
                                        .context("Failed to send followup message")?;
                                } else {
                                    unreachable!("Invalid select kind");
                                }
                            }
                            _ => unreachable!("Invalid custom id"),
                        }
                    }
                } else {
                    // Tell the user no selection has been made
                    interaction
                        .create_followup(
                            ctx,
                            CreateInteractionResponseFollowup::default().content(
                                localize_message!(
                                    "command.settings.user.response.error.noselection",
                                    &context_settings.language
                                )
                                .await
                                .context("Failed to localize message")?,
                            ),
                        )
                        .await
                        .context("Failed to send followup message")?;
                }
            }
            _ => unreachable!("Invalid custom id"),
        }
    }

    // Cleanup menu selection
    ctx.data().menu_selections.remove(&(
        reply
            .message()
            .await
            .context("Failed to get message from reply")?
            .id,
        ctx.author().id,
    ));

    Ok(())
}
