use core_data::display_types::ProjectileAddress;

pub fn address(volume: u32, name: &str) -> ProjectileAddress {
    ProjectileAddress::new(format!(
        "Assets/ThirdParty/Hovl Studio/AAA Projectiles Vol {volume}/Prefabs/Dreamtides/{name}.prefab"
    ))
}
