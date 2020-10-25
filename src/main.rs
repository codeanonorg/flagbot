use log::{debug, error, info};
use regex::Regex;
use serenity::{
    model::{
        channel::Message, event::ResumedEvent, gateway::Ready, guild::PartialGuild, id::RoleId,
    },
    prelude::*,
};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

struct GuildId;

impl TypeMapKey for GuildId {
    type Value = PartialGuild;
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Error {
    UserFacingError(String),
    InternalError(String),
}

impl From<serenity::Error> for Error {
    fn from(err: serenity::Error) -> Self {
        Self::InternalError(format!("Serenity error: {:?}", err))
    }
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

    fn help(&self) -> &'static str {
        "Tu peux rentrer ton flag comme ceci: `!flag <flag>`. Le flag est de la forme `CA{c0dE_4n0n}`. Bonne chance ! :pirate_flag:"
    }

    fn flag(&self, ctx: &Context, msg: &Message, flag: &str) -> Result<String, Error> {
        if !self.regex.is_match(flag) {
            return Err(Error::UserFacingError(
                "Le flag donné n'a pas la bonne forme. Il ressemble à ça: `CA{c0dE_4n0n}`."
                    .to_string(),
            ));
        }
        let role = self
            .flags
            .get(flag)
            .ok_or(Error::UserFacingError("Ce flag n'existe pas.".to_string()))?;
        let readref = ctx.data.read();
        let guild = readref.get::<GuildId>().unwrap();
        let role = guild
            .roles
            .get(role)
            .ok_or(Error::InternalError("Couldn't get role id".to_string()))?;
        let mut member = guild.member(&ctx.http, msg.author.id)?;
        member.add_role(&ctx.http, role)?;
        Ok(format!("Bravo, tu as trouvé le flag ! Tu as désormais le rôle `{}` sur le Discord CodeAnon ! :flag_black:", role.name))
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
            FlagCommand::handle_comand(&ctx, &msg, self.flag(&ctx, &msg, flag));
        } else if content.starts_with("!help") {
            reply(&ctx, &msg, format!(":bulb: {}", self.help()));
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

impl FlagCommand {
    fn handle_comand(ctx: &Context, msg: &Message, result: Result<String, Error>) {
        match result {
            Ok(v) => reply(&ctx, &msg, format!(":white_check_mark: {}", v)),
            Err(Error::UserFacingError(v)) => reply(&ctx, &msg, format!(":x: {}", v)),
            Err(Error::InternalError(err)) => error!("{}", err),
        }
    }
}
