use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

use crate::cost::Cost;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NamedAbility {
    /// You may play this card from your void for the given energy cost, or for
    /// its normal energy cost if none is given. If you do, banish it when it
    /// leaves play.
    Reclaim(Option<Energy>),
    /// You may play this card from your void by paying the given cost instead
    /// of paying energy. If you do, banish it when it leaves play.
    ReclaimForCost(Cost),
}
