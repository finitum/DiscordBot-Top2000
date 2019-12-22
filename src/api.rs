use serde::{Deserialize, Deserializer, de};
use crate::error::ErrorKind;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct Song {
    #[serde(rename = "aid", deserialize_with = "to_u64")]
    pub id: u64,

    #[serde(rename = "s")]
    pub title: String,

    #[serde(rename = "a")]
    pub artist: String,

    #[serde(rename = "pos")]
    pub position: u64,

}

#[derive(Debug, Clone)]
pub struct SongList {
    songs: Vec<Song>
}

pub struct NowOnAir {
    pub song: Song,
    pub img_url: Option<String>
}

fn to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error> where D: Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(de::Error::custom)
}

impl Song {
    pub fn get_description(&self) -> Result<String, ErrorKind> {
        let url = format!("https://www.nporadio2.nl/?option=com_ajax&plugin=Trackdata&format=json&songid={}", self.id);
        let body = reqwest::blocking::get(&url)
            .map_err(|e| ErrorKind::RequestError(e))?
            .text()
            .map_err(|e| ErrorKind::RequestError(e))?;

        let desc_unparsed = &serde_json::from_str::<Value>(&body).map_err(|e| ErrorKind::JsonError(e))?["data"][0]["description"];
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

        let unparsed_songs = &serde_json::from_str::<Value>(&body).map_err(|e| ErrorKind::JsonError(e))?["data"][0];
        let songs = serde_json::from_value(unparsed_songs.to_owned()).map_err(|e| ErrorKind::JsonError(e))?;
        Ok(SongList {
            songs
        })
    }

    pub fn get_now_on_air(&self) -> Result<NowOnAir, ErrorKind> {
        // temporary, as it's not top2000 yet
        self.songs.first().map(|s| NowOnAir {
            song: s.to_owned(),
            img_url: None
        }).ok_or(ErrorKind::GenericError)

//        let body = reqwest::blocking::get("https://radiobox2.omroep.nl/data/radiobox2/nowonair/2.json")
//            .map_err(|e| ErrorKind::RequestError(e))?
//            .text()
//            .map_err(|e| ErrorKind::RequestError(e))?;
//
//        let parsed_json = serde_json::from_str::<Value>(&body).map_err(|e| ErrorKind::JsonError(e))?;
//
//        let id_unparsed = &parsed_json["results"][0]["songfile"]["songversion"]["id"];
//        if let Value::Number(id) = id_unparsed {
//            if id.is_u64() {
//                let song = self.songs.iter().find(|s| s.id == id.as_u64().unwrap()).ok_or(ErrorKind::GenericError)?;
//
//                let img_url_unparsed = &parsed_json["results"][0]["songfile"]["songversion"]["image"]["url_ssl"];
//                let img_url = if let Value::String(img) = img_url_unparsed {
//                    Some(img.to_string())
//                } else {
//                    None
//                };
//
//                return Ok(NowOnAir {
//                    song: song.clone(),
//                    img_url
//                })
//            }
//        } else {
//            let artist_val = &parsed_json["results"][0]["songfile"]["artist"];
//            let title_val = &parsed_json["results"][0]["songfile"]["title"];
//
//            if let Value::String(artist) = artist_val {
//                if let Value::String(title) = title_val {
//                    let song = self.songs.iter().find(|s| {
//                        s.artist == *artist && s.title == *title
//                    });
//
//                    if song.is_some() {
//                        return Ok(NowOnAir {
//                            song: song.unwrap().to_owned(),
//                            img_url: None
//                        });
//                    } else {
//                        return Err(ErrorKind::GenericError);
//                    }
//                }
//            }
//        }
//
//        Err(ErrorKind::GenericError)
    }
}