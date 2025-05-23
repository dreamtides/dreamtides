use core_data::display_types::AudioClipAddress;

pub fn rpg_magic(pack: u32, asset: &'static str) -> AudioClipAddress {
    AudioClipAddress::new(format!(
        "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack {pack}/{asset}.wav"
    ))
}
