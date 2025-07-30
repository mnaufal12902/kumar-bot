use serenity::{client::Context, model::prelude::Message};


pub async fn run(ctx: Context, msg: Message) {
    let args = msg.content.strip_prefix("pb!ping").unwrap().trim();
    tracing::debug!("{}", args);
    if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
        println!("Error sending message: {why:?}");
    }
}
