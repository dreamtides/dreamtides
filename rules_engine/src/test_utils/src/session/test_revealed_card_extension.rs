use core_data::numerics::{Energy, Spark};
use display_data::card_view::RevealedCardView;

pub trait TestRevealedCardExtension {
    /// Returns the numeric energy cost of this card, or None if the cost is
    /// non-numeric.
    fn numeric_cost(&self) -> Option<Energy>;

    /// Returns the numeric spark value of this card, or None if the spark is
    /// non-numeric.
    fn numeric_spark(&self) -> Option<Spark>;
}

impl TestRevealedCardExtension for RevealedCardView {
    fn numeric_cost(&self) -> Option<Energy> {
        self.cost.as_ref().and_then(|s| s.parse().ok().map(Energy))
    }

    fn numeric_spark(&self) -> Option<Spark> {
        self.spark.as_ref().and_then(|s| s.parse().ok().map(Spark))
    }
}
