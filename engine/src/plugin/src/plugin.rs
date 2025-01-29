#![allow(clippy::missing_safety_doc)] // You only live once, that's the motto - Drake

pub mod test_data;

use display_data::battle_view::ClientBattleId;

#[no_mangle]
pub unsafe extern "C" fn dreamcaller_connect(
    response: *mut u8,
    response_buffer_max_length: i32,
) -> i32 {
    let scene = test_data::get_scene(ClientBattleId("123".to_string()), 0);
    let json = serde_json::to_string(&scene).unwrap();
    let json_bytes = json.as_bytes();

    if json_bytes.len() > response_buffer_max_length as usize {
        return -1;
    }

    let out = std::slice::from_raw_parts_mut(response, response_buffer_max_length as usize);
    out[..json_bytes.len()].copy_from_slice(json_bytes);
    json_bytes.len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn dreamcaller_get_scene(
    scene: i32,
    response: *mut u8,
    response_buffer_length: i32,
) -> i32 {
    let scene = test_data::get_scene(ClientBattleId("123".to_string()), scene as u32);
    let json = serde_json::to_string(&scene).unwrap();
    let json_bytes = json.as_bytes();

    if json_bytes.len() > response_buffer_length as usize {
        return -1;
    }

    let out = std::slice::from_raw_parts_mut(response, response_buffer_length as usize);
    out[..json_bytes.len()].copy_from_slice(json_bytes);
    json_bytes.len() as i32
}
