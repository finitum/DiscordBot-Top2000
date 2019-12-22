use serde::{Deserialize, Deserializer, de};
use crate::error::ErrorKind;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct Song {
    #[serde(rename = "aid", deserialize_with = "to_u64")]
    id: u64,

    #[serde(rename = "s")]
    title: String,

    #[serde(rename = "a")]
    artist: String,

    #[serde(rename = "pos")]
    position: u64
}

#[derive(Debug)]
pub struct SongList {
    songs: Vec<Song>
}

fn to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error> where D: Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(de::Error::custom)
}

impl Song {
    pub fn get_description(&self) -> Result<&String, ErrorKind> {
        let body = reqwest::blocking::get(format!("https://www.nporadio2.nl/?option=com_ajax&plugin=Trackdata&format=json&songid={}", self.id))
            .map_err(|e| ErrorKind::RequestError(e))?
            .text()
            .map_err(|e| ErrorKind::RequestError(e))?;

        let desc_unparsed = &serde_json::from_str::<Value>(&body).map_err(|e| ErrorKind::JsonError(e))?["data"][0]["description"];
        if let Value::String(desc) = desc_unparsed {
            Ok(desc)
        } else {
            Err(ErrorKind::GenericError)
        }
    }
}

impl SongList {
    pub fn new() -> Result<SongList, ErrorKind> {
        let body = include_str!("2019.json");

        println!("{}", body);

        let unparsed_songs = &serde_json::from_str::<Value>(&body).map_err(|e| ErrorKind::JsonError(e))?["data"][0];
        let songs = serde_json::from_value(unparsed_songs.to_owned()).map_err(|e| ErrorKind::JsonError(e))?;
        Ok(SongList {
            songs
        })
    }

    pub fn get_now_on_air(&self) -> Result<&Song, ErrorKind> {
        let body = reqwest::blocking::get("https://radiobox2.omroep.nl/data/radiobox2/nowonair/2.json")
            .map_err(|e| ErrorKind::RequestError(e))?
            .text()
            .map_err(|e| ErrorKind::RequestError(e))?;

        let id_unparsed = &serde_json::from_str::<Value>(&body)
            .map_err(|e| ErrorKind::JsonError(e))?["results"][0]["songfile"]["songversion"]["id"];

        if let Value::Number(id) = id_unparsed {
            if id.is_u64() {
                self.songs.iter().find(|s| s.id == id.as_u64().unwrap()).ok_or(ErrorKind::GenericError)
            } else {
                Err(ErrorKind::GenericError)
            }
        } else {
            Err(ErrorKind::GenericError)
        }
    }
}