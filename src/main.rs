use log::{debug, error, info};
use regex::Regex;
use serenity::model::event::ResumedEvent;
use serenity::{
    model::{channel::Message, gateway::Ready, prelude::*},
    prelude::*,
};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

struct GuildId;

impl TypeMapKey for GuildId {
    type Value = PartialGuild;
}

struct FlagCommand {
    regex: Regex,
    flags: HashMap<String, RoleId>,
}

impl FlagCommand {
    pub fn new(flag_file: PathBuf) -> std::io::Result<Self> {
        debug!("Reading flags from file {}", flag_file.display());
        let flag_file = std::fs::File::open(flag_file)?;
        let flag_reader = std::io::BufReader::new(flag_file);
        let mut flags = HashMap::new();
        for line in flag_reader.lines().map(|l| l.unwrap()) {
            let mut split: Vec<&str> = line.split(':').collect();
            let flag = split.remove(0);
            let role = split.remove(0).parse().expect("Parsing role id");
            flags.insert(flag.to_string(), RoleId(role));
        }
        info!("{} flags loaded.", flags.len());
        Ok(Self {
            regex: Regex::new(r"CA\{[a-zA-Z0-9_]+}").unwrap(),
            flags,
        })
    }
}

fn reply<S: AsRef<str>>(ctx: &Context, msg: &Message, content: S) {
    if let Err(err) = msg.reply(&ctx.http, content.as_ref()) {
        error!("Message reply error: {:?}", err);
    }
}

impl EventHandler for FlagCommand {
    fn message(&self, ctx: Context, msg: Message) {
        let content = msg.content.trim();
        debug!("Message: @{}: {}", msg.author.name, content);
        if content.starts_with("!flag") {
            let flag = content[5..].trim();
            if self.regex.is_match(flag) {
                if let Some(role) = self.flags.get(flag) {
                    let readref = ctx.data.read();
                    let guild = readref.get::<GuildId>().unwrap();
                    if let Some(role) = guild.roles.get(role) {
                        if let Err(err) = guild
                            .member(&ctx.http, msg.author.id)
                            .and_then(|mut m| m.add_role(&ctx.http, role.id))
                        {
                            error!("Error assigning role to member: {:?}", err);
                            reply(&ctx, &msg, format!("Tu as trouvé le flag mais malheuresement il y a eu un problème interne au serveur ... Désolé :zany_face:"))
                        } else {
                            reply(
                                &ctx,
                                &msg,
                                format!(
                                    "Tu as trouvé le flag! Tu va être assigné le rôle `{}` ! :flag_black:",
                                    role.name
                                ),
                            );
                        }
                    } else {
                        reply(&ctx, &msg, format!("Tu as trouvé le flag mais malheuresement il y a eu un problème interne au serveur ... Désolé :zany_face:"))
                    }
                } else {
                    reply(&ctx, &msg, "Ce flag n'existe pas.");
                }
            } else {
                reply(&ctx, &msg, "Le flag n'a pas la bonne forme.");
            }
        }
    }

    fn ready(&self, mut ctx: Context, data: Ready) {
        let guild = std::env::var("DISCORD_GUILD_ID")
            .ok()
            .and_then(|var| var.parse().ok())
            .and_then(|v| ctx.http.get_guild(v).ok())
            .expect("Discord guild ID");
        ctx.data.write().insert::<GuildId>(guild);
        info!("Bot ready. Connected as {}", data.user.name);
    }

    fn resume(&self, _ctx: Context, _: ResumedEvent) {
        info!("Bot resuming.")
    }
}

fn main() {
    env_logger::init();
    let token = std::env::var("DISCORD_TOKEN").expect("Discord token");
    let flag_file = PathBuf::from(std::env::var("FLAGS_FILE").expect("Flag file"));
    let mut client = Client::new(
        token,
        FlagCommand::new(flag_file).expect("Reading flag file"),
    )
    .expect("Creating client");
    client.start().expect("Client error");
}
