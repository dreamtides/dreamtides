use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A URL
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Url {
    pub url_value: String,
}

impl Url {
    pub fn new(url: String) -> Self {
        Self { url_value: url }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectileAddress {
    pub projectile: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EffectAddress {
    pub effect: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioClipAddress {
    pub audio_clip: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Milliseconds {
    pub milliseconds_value: u64,
}

impl Milliseconds {
    pub fn new(milliseconds_value: u64) -> Self {
        Self { milliseconds_value }
    }

    pub fn from_seconds(seconds: f32) -> Self {
        Self { milliseconds_value: (seconds * 1000.0) as u64 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpriteAddress {
    pub sprite: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FontAddress {
    pub font: String,
}

/// Represents a color with the given RGBA values represented as floats in the
/// 0-1 range.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisplayColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}
