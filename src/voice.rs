use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::prelude::{Mutex, TypeMapKey};
use std::sync::Arc;

pub struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}
