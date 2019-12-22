mod api;
mod error;
mod discord;
mod voice;

use api::SongList;
use crate::discord::create_bot;
use crate::error::ErrorKind;

fn main() -> Result<(), ErrorKind> {
    let song_list = SongList::new()?;
    create_bot(song_list);
    Ok(())
}
