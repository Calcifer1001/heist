use crate::PricePredictorContract;
use near_sdk::{env};

impl PricePredictorContract {
    pub fn assert_owner(&self) {
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "Only owner can call"
        );
    }
}