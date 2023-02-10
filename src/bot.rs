use dotenv::dotenv;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use teloxide::{prelude::*, utils::command::BotCommands};
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct Notif {
    id: i64,
    text: String,
    time: u64,
    chatid: String, 
}

#[derive(Debug)]
struct Notifs {
    inner: HashMap<i64, Notif>,
}

impl Notifs {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
    fn add(&mut self, notif: Notif){
        self.inner.insert(notif.id, notif);
    }

    fn next_id(&self) -> i64 {
        let mut ids: Vec<_> = self.inner.keys().collect();
        ids.sort();
        match ids.pop() {
            Some(id) => id+1,
            None => 1,
        }
    }
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    pretty_env_logger::init();
    log::info!("Starting tele bot...");

    dotenv().ok();
    let bot = Bot::from_env();

    Command::repl(bot.clone(), answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
enum Command {
    #[command(description = "display list of commands")]
    Help,
    #[command()]
    ChatId,
    #[command(description = "notify message after inserted amount of time (message seconds)", parse_with = "split")]
    Notify { text: String, time: u64 },
}

async fn send(bot: &Bot, chatide: String, text: String) -> HandlerResult {
    bot.send_message(chatide, text).await?;
    Ok(())
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let mut noti = Notifs::new();
    let chat_id = msg.chat.id;

    match cmd {
        Command::Help => 
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::ChatId => 
            bot.send_message(msg.chat.id, format!("Your chat ID: {chat_id}")).await?,
        Command::Notify { text, time } => {
            bot.send_message(msg.chat.id, notify(text, time, noti, msg.chat.id.to_string())).await?
            
            // let dur = time * 1000;
            // sleep(Duration::from_millis(dur.into())).await;
            // bot.send_message(msg.chat.id, format!("{text}")).await?
        }
        
    };

    Ok(())
}

fn notify(text: String, time: u64, mut noti: Notifs, chatid: String, ) -> String {
    let next_id = noti.next_id();
    noti.add(Notif{
        id: next_id,
        text: text,
        time: time,
        chatid: chatid
    });

    format!("Your message will be notified at {:?}", noti)
}


