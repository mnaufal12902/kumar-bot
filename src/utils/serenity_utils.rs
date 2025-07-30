use serenity::{
    builder::{CreateEmbed, CreateMessage},
    client::Context,
    model::prelude::Message,
};

pub async fn send_embed(ctx: &Context, msg: &Message, description: &str, color: u32) {
    let embed = CreateEmbed::default().description(description).color(color);

    let builder = CreateMessage::default().embed(embed);
    let _ = msg.channel_id.send_message(&ctx.http, builder).await;
}
