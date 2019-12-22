use serenity::{Client, voice};
use serenity::prelude::{EventHandler, Context, RwLock, Mutex};
use std::env;
use serenity::framework::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::model::guild::GuildStatus;
use std::cell::RefCell;
use serenity::model::channel::{GuildChannel, ChannelType};
use std::sync::{Arc};
use std::borrow::BorrowMut;
use crate::voice::VoiceManager;
use std::process::exit;
use crate::api::SongList;

type Channels = Mutex<RefCell<Vec<Arc<RwLock<GuildChannel>>>>>;

struct Handler {
    song_list: SongList,
    text_channels: Channels,
    voice_channels: Channels
}

impl Handler {
    pub fn new(song_list: SongList) -> Handler {
        Handler {
            song_list,
            text_channels: Mutex::new(RefCell::new(vec![])),
            voice_channels: Mutex::new(RefCell::new(vec![]))
        }
    }

    fn join_voice_channels(&self, ctx: Context) {
        let voice_channels = self.voice_channels.lock();

        for channel in *voice_channels.borrow() {
            let channel_locked = *channel.read();
            let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().unwrap();
            let mut manager = manager_lock.lock();
            if let Some(handler) = manager.join(channel_locked.guild_id, channel_locked.id) {
                println!("Joined channel {}!", channel_locked.name);

                let source = match voice::ytdl("https://icecast.omroep.nl/radio2-bb-mp3") {
                    Ok(source) => source,
                    Err(why) => {
                        println!("Err starting source: {:?}", why);
                        exit(1);
                    }
                };

                handler.play(source);
            } else {
                println!("Failed to join channel {}!", channel_locked.name);
                exit(1);
            }
        }
    }
}

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, data_about_bot: Ready) {
        let mut text_channels = self.text_channels.lock();
        let mut voice_channels = self.voice_channels.lock();

        for guild_status in data_about_bot.guilds {
            if let GuildStatus::OnlineGuild(guild) = guild_status {
                for channel in guild.channels {
                    let read_lock = *channel.1.read();
                    if read_lock.name == "top2000" && read_lock.kind == ChannelType::Text {
                        text_channels.borrow_mut().push(channel.1)
                    } else if read_lock.name == "top2000" && read_lock.kind == ChannelType::Voice {
                        voice_channels.borrow_mut().push(channel.1)
                    }
                }
            }
        }
    }
}

fn create_bot(song_list: SongList) -> Client {
    let handler = Handler::new(song_list);
    let mut client = Client::new(&env::var("X0xCja-HlXuAPeha9hzxPB4UiA6uSOo5").expect("token not found!"), handler).expect("error creating bot");

    let mut data = client.data.write();
    data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));

    client
        .with_framework(
            StandardFramework::new()
            .configure(|c| c.prefix("top2000-"))
        );

    client
}