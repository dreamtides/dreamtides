use core_data::display_types::MaterialAddress;

pub fn material(number: u32) -> MaterialAddress {
    MaterialAddress::new(format!("Assets/Content/Dissolves/Dissolve{number}.mat"))
}
