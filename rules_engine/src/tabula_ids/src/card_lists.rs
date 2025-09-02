use core_data::identifiers::DreamwellCardId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::uuid;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum DreamwellCardIdList {
    TestDreamwellNoAbilities,
    TestDreamwellBasic5,
}

pub fn dreamwell_card_id_list(list: DreamwellCardIdList) -> &'static [DreamwellCardId] {
    match list {
        DreamwellCardIdList::TestDreamwellNoAbilities => TEST_DREAMWELL_NO_ABILITIES,
        DreamwellCardIdList::TestDreamwellBasic5 => TEST_DREAMWELL_BASIC_5,
    }
}

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
