use core_data::display_types::{EffectAddress, ProjectileAddress};

pub fn projectile(volume: u32, name: &str) -> ProjectileAddress {
    ProjectileAddress::new(format!(
        "Assets/ThirdParty/Hovl Studio/AAA Projectiles Vol {volume}/Prefabs/Dreamtides/{name}.prefab"
    ))
}

pub fn magic_circle(name: &'static str) -> EffectAddress {
    EffectAddress::new(format!(
        "Assets/ThirdParty/Hovl Studio/Magic circles/Dreamtides/Magic circle {name}.prefab"
    ))
}
