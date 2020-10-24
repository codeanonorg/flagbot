use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::collections::{HashMap, HashSet};
use std::io::BufRead;
use std::path::PathBuf;

struct FlagCommand {
    regex: Regex,
    flags: HashMap<String, String>,
}

impl FlagCommand {
    pub fn new(flag_file: PathBuf) -> std::io::Result<Self> {
        let flag_file = std::fs::File::open(flag_file)?;
        let flag_reader = std::io::BufReader::new(flag_file);
        let mut flags = HashMap::new();
        for line in flag_reader.lines().map(|l| l.unwrap()) {
            let mut split: Vec<&str> = line.split(':').collect();
            let flag = split.remove(0);
            let role = split.remove(0);
            flags.insert(flag.to_string(), role.to_string());
        }
        Ok(Self {
            regex: Regex::new(r"CA\{[a-zA-Z0-9_]+}").unwrap(),
            flags,
        })
    }
}

fn reply<S: AsRef<str>>(ctx: Context, msg: Message, content: S) {
    if let Err(err) = msg.reply(&ctx.http, content.as_ref()) {
        eprintln!("Message reply error: {:?}", err);
    }
}

impl EventHandler for FlagCommand {
    fn message(&self, ctx: Context, msg: Message) {
        let content = msg.content.trim();
        println!("@{}: {}", msg.author.name, content);
        if content.starts_with("!flag") {
            let flag = content[5..].trim();
            if self.regex.is_match(flag) {
                if let Some(role) = self.flags.get(flag) {
                    reply(
                        ctx,
                        msg,
                        format!(
                            "Tu as trouvé le flag! Tu va être assigné le rôle `{}` !",
                            role
                        ),
                    );
                } else {
                    reply(ctx, msg, "Ce flag n'existe pas.");
                }
            } else {
                reply(ctx, msg, "Le flag n'a pas la bonne forme.");
            }
        }
    }

    fn ready(&self, _: Context, data: Ready) {
        println!("Bot ready. Connected as {}", data.user.name);
    }
}

fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Discord token");
    let flag_file = PathBuf::from(std::env::var("FLAGS_FILE").expect("Flag file"));
    let mut client = Client::new(
        token,
        FlagCommand::new(flag_file).expect("Reading flag file"),
    )
    .expect("Creating client");
    client.
    client.start().expect("Client error");
}
