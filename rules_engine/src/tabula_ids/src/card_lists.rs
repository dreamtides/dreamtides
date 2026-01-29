use core_data::identifiers::{BaseCardId, DreamwellCardId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::uuid;

pub const CORE_11: &[BaseCardId] = &[
    BaseCardId(uuid!("d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a")),
    BaseCardId(uuid!("d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a")),
    BaseCardId(uuid!("d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a")),
    BaseCardId(uuid!("d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a")),
    BaseCardId(uuid!("d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a")),
    BaseCardId(uuid!("d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a")),
    BaseCardId(uuid!("d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a")),
    BaseCardId(uuid!("d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a")),
    BaseCardId(uuid!("d07ac4fa-cc3b-4bb8-8018-de7dc1760f35")),
    BaseCardId(uuid!("d07ac4fa-cc3b-4bb8-8018-de7dc1760f35")),
    BaseCardId(uuid!("d07ac4fa-cc3b-4bb8-8018-de7dc1760f35")),
    BaseCardId(uuid!("d07ac4fa-cc3b-4bb8-8018-de7dc1760f35")),
    BaseCardId(uuid!("d07ac4fa-cc3b-4bb8-8018-de7dc1760f35")),
    BaseCardId(uuid!("d07ac4fa-cc3b-4bb8-8018-de7dc1760f35")),
    BaseCardId(uuid!("d4207a7b-fc36-45e4-a1ee-f01b34485221")),
    BaseCardId(uuid!("d4207a7b-fc36-45e4-a1ee-f01b34485221")),
    BaseCardId(uuid!("d4207a7b-fc36-45e4-a1ee-f01b34485221")),
    BaseCardId(uuid!("d4207a7b-fc36-45e4-a1ee-f01b34485221")),
    BaseCardId(uuid!("15a39336-2c71-44d6-b462-d9fd23a4d925")),
    BaseCardId(uuid!("15a39336-2c71-44d6-b462-d9fd23a4d925")),
    BaseCardId(uuid!("15a39336-2c71-44d6-b462-d9fd23a4d925")),
    BaseCardId(uuid!("15a39336-2c71-44d6-b462-d9fd23a4d925")),
    BaseCardId(uuid!("15a39336-2c71-44d6-b462-d9fd23a4d925")),
    BaseCardId(uuid!("15a39336-2c71-44d6-b462-d9fd23a4d925")),
    BaseCardId(uuid!("3380cf41-2bca-468c-95e0-57abafd29430")),
    BaseCardId(uuid!("3380cf41-2bca-468c-95e0-57abafd29430")),
    BaseCardId(uuid!("3380cf41-2bca-468c-95e0-57abafd29430")),
    BaseCardId(uuid!("3380cf41-2bca-468c-95e0-57abafd29430")),
    BaseCardId(uuid!("3380cf41-2bca-468c-95e0-57abafd29430")),
    BaseCardId(uuid!("3380cf41-2bca-468c-95e0-57abafd29430")),
    BaseCardId(uuid!("3380cf41-2bca-468c-95e0-57abafd29430")),
    BaseCardId(uuid!("3380cf41-2bca-468c-95e0-57abafd29430")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b")),
    BaseCardId(uuid!("07f737af-dbaf-471a-8edc-e1d987c23903")),
    BaseCardId(uuid!("07f737af-dbaf-471a-8edc-e1d987c23903")),
    BaseCardId(uuid!("07f737af-dbaf-471a-8edc-e1d987c23903")),
    BaseCardId(uuid!("07f737af-dbaf-471a-8edc-e1d987c23903")),
    BaseCardId(uuid!("33c0db9c-666d-4b4c-a596-b74106025be8")),
    BaseCardId(uuid!("33c0db9c-666d-4b4c-a596-b74106025be8")),
    BaseCardId(uuid!("33c0db9c-666d-4b4c-a596-b74106025be8")),
    BaseCardId(uuid!("33c0db9c-666d-4b4c-a596-b74106025be8")),
    BaseCardId(uuid!("9866955c-31af-4aad-8319-a52d2fd85d0f")),
    BaseCardId(uuid!("9866955c-31af-4aad-8319-a52d2fd85d0f")),
    BaseCardId(uuid!("9866955c-31af-4aad-8319-a52d2fd85d0f")),
    BaseCardId(uuid!("5e70988b-ce14-45a0-8334-7cf4539ee2d8")),
    BaseCardId(uuid!("5e70988b-ce14-45a0-8334-7cf4539ee2d8")),
    BaseCardId(uuid!("5e70988b-ce14-45a0-8334-7cf4539ee2d8")),
    BaseCardId(uuid!("5e70988b-ce14-45a0-8334-7cf4539ee2d8")),
    BaseCardId(uuid!("5e70988b-ce14-45a0-8334-7cf4539ee2d8")),
    BaseCardId(uuid!("5e70988b-ce14-45a0-8334-7cf4539ee2d8")),
    BaseCardId(uuid!("1d41a21a-1beb-4e56-8d7f-be29c4a9d43d")),
    BaseCardId(uuid!("1d41a21a-1beb-4e56-8d7f-be29c4a9d43d")),
    BaseCardId(uuid!("1d41a21a-1beb-4e56-8d7f-be29c4a9d43d")),
    BaseCardId(uuid!("1d41a21a-1beb-4e56-8d7f-be29c4a9d43d")),
];
pub const DREAMWELL_BASIC_5: &[DreamwellCardId] = &[
    DreamwellCardId(uuid!("bc143c3c-f149-4506-813f-1aa8dd54e370")),
    DreamwellCardId(uuid!("1dda2551-c972-4db2-81c0-9dc95ac8b27c")),
    DreamwellCardId(uuid!("dcd99f3d-8c47-47fa-9b9a-4c1455a9d2eb")),
    DreamwellCardId(uuid!("276c1e23-53d8-4aeb-a4dd-243189a44561")),
    DreamwellCardId(uuid!("63d5d9f5-208f-4e35-9681-d51a5ba6ce57")),
    DreamwellCardId(uuid!("8e5b3423-b8e1-4e1c-9e3e-5281190c4713")),
    DreamwellCardId(uuid!("e2c10a87-e92f-4170-9bbf-c288abcf9f9b")),
];
pub const TEST_DREAMWELL_BASIC_5: &[DreamwellCardId] = &[
    DreamwellCardId(uuid!("308fd4c0-ca98-4bfa-a9be-c29b36a145fd")),
    DreamwellCardId(uuid!("308fd4c0-ca98-4bfa-a9be-c29b36a145fd")),
    DreamwellCardId(uuid!("40c77ea8-a021-4bc6-8970-0853c03f3fe0")),
    DreamwellCardId(uuid!("d386663c-9e9f-4b8e-b410-f3467e39801b")),
    DreamwellCardId(uuid!("107c3b3f-6131-4ff8-afcb-f0ce4188848f")),
    DreamwellCardId(uuid!("40e4381f-12f7-46b9-ae50-67b3195781b1")),
    DreamwellCardId(uuid!("a2cdf115-8e1a-455e-a118-123f6f36c7ba")),
];
pub const TEST_DREAMWELL_NO_ABILITIES: &[DreamwellCardId] = &[
    DreamwellCardId(uuid!("308fd4c0-ca98-4bfa-a9be-c29b36a145fd")),
    DreamwellCardId(uuid!("308fd4c0-ca98-4bfa-a9be-c29b36a145fd")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2")),
];

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum BaseCardIdList {
    Core11,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum DreamwellCardIdList {
    TestDreamwellNoAbilities,
    TestDreamwellBasic5,
    DreamwellBasic5,
}

pub fn base_card_id_list(list: BaseCardIdList) -> &'static [BaseCardId] {
    match list {
        BaseCardIdList::Core11 => CORE_11,
    }
}

pub fn dreamwell_card_id_list(list: DreamwellCardIdList) -> &'static [DreamwellCardId] {
    match list {
        DreamwellCardIdList::TestDreamwellNoAbilities => TEST_DREAMWELL_NO_ABILITIES,
        DreamwellCardIdList::TestDreamwellBasic5 => TEST_DREAMWELL_BASIC_5,
        DreamwellCardIdList::DreamwellBasic5 => DREAMWELL_BASIC_5,
    }
}
