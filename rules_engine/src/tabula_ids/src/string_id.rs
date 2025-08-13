use tabula::localized_strings::StringId;
use uuid::uuid;

/// Energy icon
pub const E: StringId = StringId(uuid!("dbb75f1b-8c25-4c27-a598-7b300f5b5ca4"));

/// Fast icon
pub const F: StringId = StringId(uuid!("7b21533d-7f49-451c-bfa1-949ab28ed258"));

/// Activated icon
pub const A: StringId = StringId(uuid!("adc9a41d-fd82-49b5-a099-a9f10a73afad"));

/// Multi-activated icon
pub const MA: StringId = StringId(uuid!("121fc3af-6abd-4a90-9197-43c6e18eeca2"));

/// Foresee keyword ability
pub const FORESEE: StringId = StringId(uuid!("2e44ee4c-3218-45bf-a29e-0508f853c873"));

/// Reclaim keyword ability
pub const RECLAIM: StringId = StringId(uuid!("fb895ce7-1f66-426d-8c29-b19fdeee7828"));

/// Kindle keyword ability
pub const KINDLE: StringId = StringId(uuid!("bf189fe0-75e2-4a87-9dca-67dd5f755766"));

/// Decline to take the action associated with a prompt
pub const DECLINE_PROMPT_BUTTON: StringId = StringId(uuid!("6095730f-d43c-49cd-a5dc-2781882389ed"));

/// Choose to pay energy to take a prompt action
pub const PAY_ENERGY_PROMPT_BUTTON: StringId =
    StringId(uuid!("211e9d51-07ed-4261-88ce-fbfeb3390449"));
