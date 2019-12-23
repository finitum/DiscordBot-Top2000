use serenity::{Client, voice};
use serenity::prelude::{EventHandler, Context, Mutex};
use serenity::model::gateway::{Ready, Activity};
use std::cell::RefCell;
use serenity::model::channel::{GuildChannel, ChannelType};
use std::sync::{Arc};
use crate::voice::VoiceManager;
use crate::api::{SongList, NowOnAir};
use serenity::framework::standard::StandardFramework;
use dotenv::dotenv;
use std::thread;
use serenity::model::user::OnlineStatus;
use chrono::Utc;
use chrono::Duration;
use std::time::Duration as StdDuration;
use std::env;

type Channels = Mutex<RefCell<Vec<GuildChannel>>>;

struct Handler {
    song_list: SongList,
    text_channels: Channels,
    voice_channels: Channels,
    now_on_air: Option<NowOnAir>
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        let text_channels_lock = self.text_channels.lock();
        let voice_channels_lock = self.voice_channels.lock();

        Handler {
            song_list: self.song_list.clone(),
            text_channels: Mutex::new(text_channels_lock.clone()),
            voice_channels: Mutex::new(voice_channels_lock.clone()),
            now_on_air: self.now_on_air.clone()
        }
    }
}

impl Handler {
    pub fn new(song_list: SongList) -> Handler {
        Handler {
            song_list,
            text_channels: Mutex::new(RefCell::new(vec![])),
            voice_channels: Mutex::new(RefCell::new(vec![])),
            now_on_air: None
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
        let mut self_clone = self.clone();
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            loop {
                let now_on_air = self_clone.song_list.get_now_on_air();

                if let Ok(on_air) = now_on_air {
                    let prev_now_on_air = self_clone.now_on_air.as_ref();
                    if prev_now_on_air.is_none() || on_air.song.id != prev_now_on_air.unwrap().song.id {
                        self_clone.now_on_air = Some(on_air.clone());

                        let title = &on_air.song.title;
                        println!("New song: {}", title);

                        self_clone.update_presence(&ctx_clone, &on_air);
                        self_clone.generate_embed(&ctx_clone, &on_air);

                        let diff = (on_air.end_time - Utc::now()) - Duration::seconds(15);
                        thread::sleep(diff.to_std().unwrap_or(StdDuration::from_secs(15)));

                        continue;
                    }
                } else {
                    println!("Getting now on air failed miserably!");
                }

                thread::sleep(Duration::seconds(15).to_std().unwrap_or(StdDuration::from_secs(15)));
            }
        });
    }

    fn generate_embed(&self, ctx: &Context, now_on_air: &NowOnAir) {

        let text_channels = self.text_channels.lock();
        let text_ref = &*text_channels.borrow();

        for text_channel in text_ref {
            let _ = text_channel.send_message(ctx, |m| {
                m.embed(|e| {
                    e
                        .title(format!("{} by {}", now_on_air.song.title, now_on_air.song.artist))
                        .description(now_on_air.song.get_description().unwrap_or_else(|_| "".to_string()))
                        .image(now_on_air.img_url.as_ref().unwrap_or(&"".to_string()))
                        .field("Position", now_on_air.song.position.map_or("unknown".to_string(), |f| f.to_string()), false)
                        .url(format!("https://www.nporadio2.nl{}", now_on_air.song.url))
                })
            });
        }

    }

    fn update_presence(&self, ctx: &Context, now_on_air: &NowOnAir) {
        let activity = Activity::listening(format!("{} by {}", now_on_air.song.title, now_on_air.song.artist).as_ref());
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
                let channels_res = ctx.http.get_channels(guild.id.0);

                if let Ok(channels) = channels_res {
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
        }

        self.join_voice_channels(&ctx);
        self.background_loop(&ctx);
    }
}

pub fn create_bot(song_list: SongList) {
    let handler = Handler::new(song_list);

    let _ = dotenv();
    let env_token = env::var("DISCORD_TOKEN");
    if let Err(_) = env_token {
        panic!("Environment variable not found!")
    }

    let mut client = Client::new(env_token.unwrap(), handler).expect("error creating bot");

    client
        .with_framework(
            StandardFramework::new()
                .configure(|c| c.prefix("top2000-"))
        );

    client.data.write().insert::<VoiceManager>(Arc::clone(&client.voice_manager));

    let _ = client.start().map_err(|why| println!("Client ended: {:?}", why));
}