use serenity::prelude::{TypeMapKey, Mutex};
use std::sync::{Arc};
use serenity::client::bridge::voice::ClientVoiceManager;

pub struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}