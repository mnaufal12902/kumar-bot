use serenity::{
    all::{
        ChannelType, CreateChannel, CreateEmbedFooter, EditRole, Guild, GuildChannel, ImageHash,
        PermissionOverwrite, PermissionOverwriteType, Permissions, User, UserId,
    },
    builder::{CreateEmbed, CreateMessage},
    client::Context,
    model::prelude::Message,
};

pub async fn send_embed(
    ctx: &Context,
    msg: &Message,
    description: &str,
    color: u32,
) -> serenity::Result<Message> {
    let embed = CreateEmbed::default().description(description).color(color);

    let builder = CreateMessage::default().embed(embed);
    msg.channel_id.send_message(&ctx.http, builder).await
}

pub async fn send_track_embed(
    ctx: &Context,
    msg: &Message,
    title: &str,
    artist: &str,
    duration: &u32,
    thumbnail: &str,
    cur_index: &str,
    index: &str,
    user: User,
) -> serenity::Result<Message> {
    let url_picture = match user.avatar {
        Some(hash) => {
            let hash_str = hash.to_string();
            let ext = if hash.is_animated() { "gif" } else { "png" };
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.{}",
                user.id, hash_str, ext
            )
        }
        None => {
            format!("https://cdn-icons-png.flaticon.com/512/747/747545.png")
        }
    };

    let duration_str = {
        let total_seconds = duration / 1000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    };

    let embed = CreateEmbed::new()
        .title("Added Track Queue")
        .thumbnail(thumbnail)
        .fields([
            ("Track     ", title, true),
            ("Artist    ", artist, true),
            ("Track Length  ", &duration_str, true),
            ("Current position", cur_index, true),
            ("Position in queue", index, true),
        ])
        .footer(CreateEmbedFooter::new(format!("Requested by {}", user.name)).icon_url(url_picture))
        .color(0x00AAFF);

    let builder = CreateMessage::default().embed(embed);
    msg.channel_id.send_message(&ctx.http, builder).await
}

pub async fn create_channel_from_guild(
    ctx: Context,
    guild: Guild,
) -> serenity::Result<GuildChannel> {
    let category_name = r"Bot Channels";
    let channel_name = r"kumar-channel";
    let allowed_role_name = r"Kumar"; // Just this role can be use this bot

    let allowed_role = match guild.role_by_name(&allowed_role_name) {
        Some(r) => r.to_owned(),
        None => {
            let role = guild
                .id
                .create_role(
                    &ctx.http,
                    EditRole::new()
                        .name(allowed_role_name)
                        .colour((241, 197, 102))
                        .permissions(Permissions::SEND_MESSAGES)
                        .mentionable(true),
                )
                .await?;

            role
        }
    };

    let channels = guild.id.channels(&ctx.http).await?;

    let category = channels
        .values()
        .find(|f| f.kind == ChannelType::Category && f.name == category_name);

    let category_id = if let Some(cat) = category {
        cat.id
    } else {
        let new_cat = guild
            .id
            .create_channel(
                &ctx.http,
                CreateChannel::new(category_name).kind(ChannelType::Category),
            )
            .await?;

        new_cat.id
    };

    let channel_exists = channels.values().any(|c| {
        c.kind == ChannelType::Text && c.name == channel_name && c.parent_id == Some(category_id)
    });

    let channel = if !channel_exists {
        guild
            .id
            .create_channel(
                &ctx.http,
                CreateChannel::new(channel_name)
                    .kind(ChannelType::Text)
                    .category(category_id)
                    .permissions([
                        // Deny from everyone
                        PermissionOverwrite {
                            allow: Permissions::empty(),
                            deny: Permissions::SEND_MESSAGES,
                            kind: PermissionOverwriteType::Role(guild.id.everyone_role()),
                        },
                        // Allow for target role
                        PermissionOverwrite {
                            allow: Permissions::all(),
                            deny: Permissions::empty(),
                            kind: PermissionOverwriteType::Role(allowed_role.id),
                        },
                    ]),
            )
            .await?
    } else {
        guild
            .channels(&ctx.http)
            .await?
            .values()
            .find(|c| c.name == channel_name)
            .cloned()
            .ok_or_else(|| serenity::Error::Other("Channel not found"))?
    };

    Ok(channel)
}

pub async fn create_channel_from_id(
    ctx: &Context,
    msg: &Message,
) -> serenity::Result<GuildChannel> {
    let category_name = r"Bot Channels";
    let channel_name = r"kumar-channel";
    let allowed_role_name = r"Kumar"; // Just this role can be use this bot

    let guild_id = msg.guild_id.ok_or(serenity::Error::Model(
        serenity::all::ModelError::GuildNotFound,
    ))?;

    let roles = guild_id.roles(&ctx.http).await?;

    let allowed_role = match roles.values().find(|v| v.name == allowed_role_name) {
        Some(r) => r.to_owned(),
        None => {
            let role = guild_id
                .create_role(
                    &ctx.http,
                    EditRole::new()
                        .name(allowed_role_name)
                        .colour((241, 197, 102))
                        .permissions(Permissions::SEND_MESSAGES)
                        .mentionable(true),
                )
                .await?;
            role
        }
    };

    let channels = guild_id.channels(&ctx.http).await?;

    let category = channels
        .values()
        .find(|f| f.kind == ChannelType::Category && f.name == category_name);

    let category_id = if let Some(cat) = category {
        cat.id
    } else {
        let new_cat = guild_id
            .create_channel(
                &ctx.http,
                CreateChannel::new(category_name).kind(ChannelType::Category),
            )
            .await?;

        new_cat.id
    };

    let channel_exists = channels.values().any(|c| {
        c.kind == ChannelType::Text && c.name == channel_name && c.parent_id == Some(category_id)
    });

    let channel = if !channel_exists {
        guild_id
            .create_channel(
                &ctx.http,
                CreateChannel::new(channel_name)
                    .kind(ChannelType::Text)
                    .category(category_id)
                    .permissions([
                        // Deny from everyone
                        PermissionOverwrite {
                            allow: Permissions::empty(),
                            deny: Permissions::SEND_MESSAGES,
                            kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
                        },
                        // Allow for target role
                        PermissionOverwrite {
                            allow: Permissions::all(),
                            deny: Permissions::empty(),
                            kind: PermissionOverwriteType::Role(allowed_role.id),
                        },
                    ]),
            )
            .await?
    } else {
        channels
            .values()
            .find(|c| c.name == channel_name)
            .cloned()
            .ok_or_else(|| serenity::Error::Other("Channel not found"))?
    };

    Ok(channel)
}
