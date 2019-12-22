use serenity::{Client, voice};
use serenity::prelude::{EventHandler, Context, Mutex};
use serenity::model::gateway::{Ready, Activity};
use std::cell::RefCell;
use serenity::model::channel::{GuildChannel, ChannelType};
use std::sync::{Arc};
use crate::voice::VoiceManager;
use crate::api::{SongList, Song, NowOnAir};
use serenity::framework::standard::StandardFramework;
use dotenv_codegen::dotenv;
use std::thread;
use std::time::Duration;
use serenity::model::user::OnlineStatus;

type Channels = Mutex<RefCell<Vec<GuildChannel>>>;

struct Handler {
    song_list: SongList,
    text_channels: Channels,
    voice_channels: Channels,
    current_song: Option<Song>
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        let text_channels_lock = self.text_channels.lock();
        let voice_channels_lock = self.voice_channels.lock();

        Handler {
            song_list: self.song_list.clone(),
            text_channels: Mutex::new(text_channels_lock.clone()),
            voice_channels: Mutex::new(voice_channels_lock.clone()),
            current_song: self.current_song.clone()
        }
    }
}

impl Handler {
    pub fn new(song_list: SongList) -> Handler {
        let song = song_list.get_now_on_air().ok().map(|s| s.song.clone());

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
            let joined_channel = manager.join(channel.guild_id, channel.id);
            if let Some(handler) = joined_channel {
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

    fn background_loop(&self, ctx: &Context) {
        let self_clone = self.clone();
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            loop {
                self_clone.generate_embed(&ctx_clone);
                thread::sleep(Duration::from_millis(15000));
            }
        });
    }

    fn generate_embed(&self, ctx: &Context) {
        let now_on_air = self.song_list.get_now_on_air().unwrap();

        let title = &now_on_air.song.title;
        let desc = now_on_air.song.get_description().unwrap();
        println!("{} with desc {}", title, desc);

        self.update_presence(ctx, &now_on_air);
    }

    fn update_presence(&self, ctx: &Context, now_on_air: &NowOnAir) {
        let mut activity = Activity::listening(format!("{} by {}", now_on_air.song.title, now_on_air.song.artist).as_ref());

        let img = if now_on_air.img_url.is_some() {
            now_on_air.img_url.clone()
        } else {
            Some("https://i.imgur.com/Z3yujMQ.png".to_string())
        };

        activity.url = img;
        ctx.set_presence(Some(activity), OnlineStatus::Online);
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
        self.background_loop(&ctx);
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