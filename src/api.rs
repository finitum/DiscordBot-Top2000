use serde::{Deserialize};
use crate::error::ErrorKind;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Song {
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
}