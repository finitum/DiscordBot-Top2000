use serde::{Deserialize, Deserializer, de};
use crate::error::ErrorKind;
use serde_json::Value;
use chrono::{DateTime, Utc};
use std::str::FromStr;

#[derive(Debug, Deserialize, Clone)]
pub struct Song {
    #[serde(rename = "aid", deserialize_with = "to_u64")]
    pub id: u64,

    #[serde(rename = "s")]
    pub title: String,

    #[serde(rename = "a")]
    pub artist: String,

    #[serde(rename = "pos")]
    pub position: Option<u64>,

    #[serde(rename = "url")]
    pub url: String

}

#[derive(Debug, Clone)]
pub struct SongList {
    songs: Vec<Song>
}

#[derive(Debug, Clone)]
pub struct NowOnAir {
    pub song: Song,
    pub img_url: Option<String>,
    pub end_time: DateTime<Utc>
}

fn to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error> where D: Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(de::Error::custom)
}

impl Song {
    pub fn get_description(&self) -> Result<String, ErrorKind> {
        let url = format!("https://www.nporadio2.nl/?option=com_ajax&plugin=Trackdata&format=json&songid={}", self.id);
        let body = reqwest::blocking::get(&url)
            .map_err(ErrorKind::RequestError)?
            .text()
            .map_err(ErrorKind::RequestError)?;

        let desc_unparsed = &serde_json::from_str::<Value>(&body).map_err(ErrorKind::JsonError)?["data"][0]["description"];
        if let Value::String(desc) = desc_unparsed {
            Ok(desc.to_owned())
        } else {
            Err(ErrorKind::GenericError)
        }
    }
}

impl SongList {
    pub fn new() -> Result<SongList, ErrorKind> {
        let body = include_str!("2019.json");

        println!("{}", body);

        let unparsed_songs = &serde_json::from_str::<Value>(&body).map_err(ErrorKind::JsonError)?["data"][0];
        let songs = serde_json::from_value(unparsed_songs.to_owned()).map_err(ErrorKind::JsonError)?;
        Ok(SongList {
            songs
        })
    }

    pub fn get_now_on_air(&self) -> Result<NowOnAir, ErrorKind> {

        let body = reqwest::blocking::get("https://radiobox2.omroep.nl/data/radiobox2/nowonair/2.json")
            .map_err(ErrorKind::RequestError)?
            .text()
            .map_err(ErrorKind::RequestError)?;

        let parsed_json = serde_json::from_str::<Value>(&body).map_err( ErrorKind::JsonError)?;

        let end_time_unparsed = &parsed_json["results"][0]["stopdatetime"];
        let end_time = if let Value::String(end_time_str) = end_time_unparsed {
            DateTime::from_str(end_time_str).map_err(|_e| ErrorKind::GenericError)?
        } else {
            return Err(ErrorKind::GenericError);
        };

        let id_unparsed = &parsed_json["results"][0]["songfile"]["songversion"]["id"];
        if let Value::Number(id) = id_unparsed {
            if let Some(id_unwrapped) = id.as_u64() {
                let song_option = self.songs.iter().find(|s| s.id == id_unwrapped);
                if let Some(song) = song_option {
                    let img_url_unparsed = &parsed_json["results"][0]["songfile"]["songversion"]["image"]["url_ssl"];
                    let img_url = if let Value::String(img) = img_url_unparsed {
                        Some(img.to_string())
                    } else {
                        None
                    };

                    return Ok(NowOnAir {
                        song: song.clone(),
                        img_url,
                        end_time
                    })
                }
            }
        }

        let artist_val = &parsed_json["results"][0]["songfile"]["artist"];
        let title_val = &parsed_json["results"][0]["songfile"]["title"];

        if let Value::String(artist) = artist_val {
            if let Value::String(title) = title_val {
                let song = self.songs.iter().find(|s| {
                    s.artist == *artist && s.title == *title
                });

                return match song {
                    Some(song_some) => Ok(NowOnAir {
                        song: song_some.to_owned(),
                        img_url: None,
                        end_time
                    }),
                    None => Ok(NowOnAir {
                        song: Song {
                            id: 0,
                            title: title.to_string(),
                            artist: artist.to_string(),
                            position: None,
                            url: "".to_string()
                        },
                        img_url: None,
                        end_time
                    })
                }
            }
        }

        Err(ErrorKind::GenericError)
    }
}