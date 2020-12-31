use crate::error::ErrorKind;
use serde::{de, Deserialize, Deserializer};
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
    pub position: Option<u64>,

    #[serde(rename = "prv")]
    pub prev_position: Option<u64>,

    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "img")]
    pub image: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SongList {
    songs: Vec<Song>,
}

#[derive(Debug, Clone)]
pub struct NowOnAir {
    pub song: Song,
    pub img_url: Option<String>,
}

fn to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(de::Error::custom)
}

impl Song {
    pub async fn get_description(&self) -> Result<String, ErrorKind> {
        let url = format!(
            "https://www.nporadio2.nl/?option=com_ajax&plugin=Trackdata&format=json&songid={}",
            self.id
        );
        let body = reqwest::get(&url)
            .await
            .map_err(ErrorKind::RequestError)?
            .text()
            .await
            .map_err(ErrorKind::RequestError)?;

        let desc_unparsed = &serde_json::from_str::<Value>(&body).map_err(ErrorKind::JsonError)?
            ["data"][0]["description"];
        if let Value::String(desc) = desc_unparsed {
            Ok(desc.to_owned())
        } else {
            Err(ErrorKind::GenericError)
        }
    }

    pub fn get_last_year_position(&self) -> Option<u64> {
        self.prev_position
    }
}

impl SongList {
    pub fn new() -> Result<SongList, ErrorKind> {
        let body = include_str!("2020.json");

        let unparsed_songs =
            &serde_json::from_str::<Value>(&body).map_err(ErrorKind::JsonError)?["data"][0];
        let songs: Vec<_> =
            serde_json::from_value(unparsed_songs.to_owned()).map_err(ErrorKind::JsonError)?;

        println!("Successfully parsed {} songs!", &songs.len());

        Ok(SongList { songs })
    }

    pub async fn get_now_on_air(&self) -> Result<NowOnAir, ErrorKind> {
        let body =
            reqwest::get("https://www.nporadio2.nl/?option=com_ajax&plugin=nowplaying&format=json&channel=nporadio2").await
                .map_err(ErrorKind::RequestError)?
                .text().await
                .map_err(ErrorKind::RequestError)?;

        let parsed_json = serde_json::from_str::<Value>(&body).map_err(ErrorKind::JsonError)?;

        let id_unparsed = &parsed_json["data"][0]["id"];
        if let Value::String(id) = id_unparsed {
            if let Ok(id_unwrapped) = id.parse::<u64>() {
                let song_option = self.songs.iter().find(|s| s.id == id_unwrapped);
                if let Some(song) = song_option {
                    let img_url_unparsed = &parsed_json["data"][0]["image"];
                    let img_url = if let Value::String(img) = img_url_unparsed {
                        Some(img.to_string())
                    } else {
                        None
                    };

                    return Ok(NowOnAir {
                        song: song.clone(),
                        img_url,
                    });
                }
            }
        }

        let artist_val = &parsed_json["data"][0]["artist"];
        let title_val = &parsed_json["data"][0]["title"];

        if let Value::String(artist) = artist_val {
            if let Value::String(title) = title_val {
                let song = self
                    .songs
                    .iter()
                    .find(|s| s.artist == *artist && s.title == *title);

                return match song {
                    Some(song_some) => Ok(NowOnAir {
                        song: song_some.to_owned(),
                        img_url: song_some.image.clone(),
                    }),
                    None => Ok(NowOnAir {
                        song: Song {
                            id: 0,
                            title: title.to_string(),
                            artist: artist.to_string(),
                            position: None,
                            url: "".to_string(),
                            image: None,
                            prev_position: None,
                        },
                        img_url: None,
                    }),
                };
            }
        }

        Err(ErrorKind::GenericError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let s_list = SongList::new().unwrap();
        let res = s_list.get_now_on_air().await.unwrap();
        dbg!(res);
    }
}
