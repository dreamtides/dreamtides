use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Trait for simple types which can be converted to and from strings.
pub trait StringWrapper: Sized {
    fn to_string_value(&self) -> String;
    fn from_string_value(s: &str) -> Result<Self, String>;
}

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

impl StringWrapper for Url {
    fn to_string_value(&self) -> String {
        self.url_value.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(Url { url_value: s.to_string() })
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

impl StringWrapper for ProjectileAddress {
    fn to_string_value(&self) -> String {
        self.projectile.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(ProjectileAddress { projectile: s.to_string() })
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

impl StringWrapper for EffectAddress {
    fn to_string_value(&self) -> String {
        self.effect.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(EffectAddress { effect: s.to_string() })
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

impl StringWrapper for AudioClipAddress {
    fn to_string_value(&self) -> String {
        self.audio_clip.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(AudioClipAddress { audio_clip: s.to_string() })
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

impl StringWrapper for TextureAddress {
    fn to_string_value(&self) -> String {
        self.texture.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(TextureAddress { texture: s.to_string() })
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

impl StringWrapper for SpriteAddress {
    fn to_string_value(&self) -> String {
        self.sprite.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(SpriteAddress { sprite: s.to_string() })
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

impl StringWrapper for PrefabAddress {
    fn to_string_value(&self) -> String {
        self.prefab.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(PrefabAddress { prefab: s.to_string() })
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

impl StringWrapper for MaterialAddress {
    fn to_string_value(&self) -> String {
        self.material.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(MaterialAddress { material: s.to_string() })
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

impl StringWrapper for FontAddress {
    fn to_string_value(&self) -> String {
        self.font.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(FontAddress { font: s.to_string() })
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

impl StringWrapper for Milliseconds {
    fn to_string_value(&self) -> String {
        let ms = self.milliseconds_value;
        let mut s = format!("{:.3}", ms as f32 / 1000.0);
        while s.contains('.') && s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
        s
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        let trimmed = s.trim();
        let seconds = trimmed.parse::<f32>().map_err(|e| e.to_string())?;
        if seconds < 0.0 {
            return Err("negative seconds".to_string());
        }
        let ms = (seconds * 1000.0).round() as u32;
        Ok(Milliseconds { milliseconds_value: ms })
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

impl StringWrapper for StudioAnimation {
    fn to_string_value(&self) -> String {
        self.name.clone()
    }

    fn from_string_value(s: &str) -> Result<Self, String> {
        Ok(StudioAnimation { name: s.to_string() })
    }
}
