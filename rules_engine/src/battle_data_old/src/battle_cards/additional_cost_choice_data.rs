use core_data::numerics::Energy;

/// Represents a choice of additional cost for a card.
#[derive(Debug, Clone)]
pub enum AdditionalCostData {
    PayEnergy(Energy),
}
