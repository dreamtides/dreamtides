use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A URL
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Url {
    pub url_value: String,
}

impl Url {
    pub fn new(url: String) -> Self {
        Self { url_value: url }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectileAddress {
    pub projectile: String,
}

impl ProjectileAddress {
    pub fn new(projectile: impl Into<String>) -> Self {
        Self { projectile: projectile.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EffectAddress {
    pub effect: String,
}

impl EffectAddress {
    pub fn new(effect: impl Into<String>) -> Self {
        Self { effect: effect.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AudioClipAddress {
    pub audio_clip: String,
}

impl AudioClipAddress {
    pub fn new(audio_clip: impl Into<String>) -> Self {
        Self { audio_clip: audio_clip.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TextureAddress {
    pub texture: String,
}

impl TextureAddress {
    pub fn new(texture: impl Into<String>) -> Self {
        Self { texture: texture.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SpriteAddress {
    pub sprite: String,
}

impl SpriteAddress {
    pub fn new(sprite: impl Into<String>) -> Self {
        Self { sprite: sprite.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PrefabAddress {
    pub prefab: String,
}

impl PrefabAddress {
    pub fn new(prefab: impl Into<String>) -> Self {
        Self { prefab: prefab.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct MaterialAddress {
    pub material: String,
}

impl MaterialAddress {
    pub fn new(material: impl Into<String>) -> Self {
        Self { material: material.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FontAddress {
    pub font: String,
}

impl FontAddress {
    pub fn new(font: impl Into<String>) -> Self {
        Self { font: font.into() }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Milliseconds {
    pub milliseconds_value: u32,
}

impl Milliseconds {
    pub fn new(milliseconds_value: u32) -> Self {
        Self { milliseconds_value }
    }

    pub fn from_seconds(seconds: f32) -> Self {
        Self { milliseconds_value: (seconds * 1000.0) as u32 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct StudioAnimation {
    pub name: String,
}

impl StudioAnimation {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
