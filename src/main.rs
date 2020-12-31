mod api;
mod discord;
mod error;
mod voice;

use crate::discord::create_bot;
use crate::error::ErrorKind;
use api::SongList;

#[tokio::main]
async fn main() -> Result<(), ErrorKind> {
    let song_list = SongList::new()?;
    create_bot(song_list).await;
    Ok(())
}
