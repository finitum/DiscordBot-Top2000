use serenity::{Client, voice};
use serenity::prelude::{EventHandler, Context, Mutex};
use serenity::model::gateway::Ready;
use std::cell::RefCell;
use serenity::model::channel::{GuildChannel, ChannelType};
use std::sync::{Arc};
use crate::voice::VoiceManager;
use crate::api::{SongList, Song};
use serenity::framework::standard::StandardFramework;
use dotenv_codegen::dotenv;
use std::thread;
use std::time::Duration;

type Channels = Mutex<RefCell<Vec<GuildChannel>>>;

struct Handler {
    song_list: SongList,
    text_channels: Channels,
    voice_channels: Channels,
    current_song: Option<Song>
}

impl Handler {
    pub fn new(song_list: SongList) -> Handler {
        let song = song_list.get_now_on_air().ok().map(|s| s.clone());

        Handler {
            song_list,
            text_channels: Mutex::new(RefCell::new(vec![])),
            voice_channels: Mutex::new(RefCell::new(vec![])),
            current_song: song
        }
    }

    fn join_voice_channels(&self, ctx: &Context) {
        let voice_channels = self.voice_channels.lock();
        let voice_ref = &*voice_channels.borrow();

        let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().unwrap();
        let mut manager = manager_lock.lock();

        for channel in voice_ref {
            if let Some(handler) = manager.join(channel.guild_id, channel.id) {
                println!("Joined channel top2000 on server {}!", channel.guild_id);

                let source = match voice::ytdl("https://icecast.omroep.nl/radio2-bb-mp3") {
                    Ok(source) => source,
                    Err(why) => {
                        panic!("Err starting source: {:?}", why);
                    }
                };

                handler.play(source);
            } else {
                panic!("Failed to join channel on server {}!", channel.guild_id);
            }
        }
    }
}

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        {
            let text_channels = self.text_channels.lock();
            let voice_channels = self.voice_channels.lock();

            let guard = ctx.cache.read();
            let guilds_res = &guard.user.guilds(&ctx.http);
            let guilds = if let Ok(guild) = guilds_res {
                guild
            } else {
                panic!("No guilds found!");
            };

            for guild in guilds {
                let channels = ctx.http.get_channels(guild.id.0).expect("No channels found!");
                for channel in channels {
                    if channel.name == "top2000" && channel.kind == ChannelType::Text {
                        let ref_vec = &*text_channels;
                        ref_vec.borrow_mut().push(channel);
                    } else if channel.name == "top2000" && channel.kind == ChannelType::Voice {
                        let ref_vec = &*voice_channels;
                        ref_vec.borrow_mut().push(channel);
                    }
                }
            }
        }

        self.join_voice_channels(&ctx);
        thread::spawn(move || {
            loop {
                println!("hi!");
                thread::sleep(Duration::from_millis(5000));
            }
        });
    }
}

pub fn create_bot(song_list: SongList) {
    let handler = Handler::new(song_list);

    let env_token = dotenv!("DISCORD_TOKEN");
    let mut client = Client::new(env_token, handler).expect("error creating bot");

    client
        .with_framework(
            StandardFramework::new()
                .configure(|c| c.prefix("top2000-"))
        );

    client.data.write().insert::<VoiceManager>(Arc::clone(&client.voice_manager));

    let _ = client.start().map_err(|why| println!("Client ended: {:?}", why));
}