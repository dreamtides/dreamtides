use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use core_data::identifiers::UserId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisplayProperties {
    pub screen_width: u32,
    pub screen_height: u32,
    pub is_mobile_device: bool,
}

impl Default for DisplayProperties {
    fn default() -> Self {
        Self { screen_width: 1920, screen_height: 1080, is_mobile_device: false }
    }
}

static USER_DISPLAY_PROPERTIES: LazyLock<Mutex<HashMap<UserId, DisplayProperties>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static MOST_RECENT_DISPLAY_PROPERTIES: LazyLock<Mutex<Option<DisplayProperties>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn store_display_properties(user_id: UserId, properties: DisplayProperties) {
    let mut display_props = USER_DISPLAY_PROPERTIES.lock().unwrap();
    display_props.insert(user_id, properties.clone());

    let mut recent_props = MOST_RECENT_DISPLAY_PROPERTIES.lock().unwrap();
    *recent_props = Some(properties);
}

pub fn get_display_properties() -> DisplayProperties {
    let recent_props = MOST_RECENT_DISPLAY_PROPERTIES.lock().unwrap();
    recent_props.clone().unwrap_or_default()
}

pub fn get_display_properties_for_user(user_id: UserId) -> DisplayProperties {
    let display_props = USER_DISPLAY_PROPERTIES.lock().unwrap();
    display_props.get(&user_id).cloned().unwrap_or_default()
}
