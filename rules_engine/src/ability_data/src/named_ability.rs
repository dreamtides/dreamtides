use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NamedAbility {
    /// You may play this card from your void for the given energy cost, or for
    /// its normal energy cost if none is given. If you do, banish it when it
    /// leaves play.
    Reclaim(Option<Energy>),
}
