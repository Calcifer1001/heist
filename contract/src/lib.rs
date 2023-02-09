// Find all our documentation at https://docs.near.org
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, log, near_bindgen, Balance, AccountId};

const EPOCH_PRICE_MULTIPLIER: u128 = 1000125079u128;
const INITIAL_BALANCE: u128 = 100_000_000_000_000_000_000_000_000;

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Bet {
    token: AccountId,
    initial_token_price: Balance,
    in_token: i8,
    amount: Balance,
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct PricePredictorContract {
    owner: AccountId,
    registered_accounts: Vec<AccountId>,
    current_prices: UnorderedMap<AccountId, Balance>,
    current_bets: UnorderedMap<AccountId, Bet>,
    heist_balances: UnorderedMap<AccountId, Balance>,
    stheist_balances: UnorderedMap<AccountId, Balance>,
    stheist_price: Balance,
}

// Define the default, which automatically initializes the contract
// impl Default for PricePredictorContract {
//     fn default() -> Self{
//         Self{
//             owner: "contract".parse().unwrap(),
//             registered_accounts: Vec::new(),
//             current_prices: UnorderedMap::new(b"current_prices".to_vec()),
//             current_bets: UnorderedMap::new(b"current_bets".to_vec()),
//             heist_balances: UnorderedMap::new(b"heist_balances".to_vec()),
//             stheist_balances: UnorderedMap::new(b"stheist_balances".to_vec()),
//             stheist_price: 1_000_000_000_000u128,
//         }
//     }
// }


// Implement the contract structure
#[near_bindgen]
impl PricePredictorContract {

    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner: owner_id,
            registered_accounts: Vec::new(),
            current_prices: UnorderedMap::new(b"current_prices".to_vec()),
            current_bets: UnorderedMap::new(b"current_bets".to_vec()),
            heist_balances: UnorderedMap::new(b"heist_balances".to_vec()),
            stheist_balances: UnorderedMap::new(b"stheist_balances".to_vec()),
            stheist_price: 1_000_000_000_000u128,
        }
    }

    pub fn register(&mut self, bet_token_id: i8) {
        let mut ini_balance = INITIAL_BALANCE;
        if bet_token_id == 1i8 {
            ini_balance = INITIAL_BALANCE * 1_000_000_000_000u128/ self.stheist_price;
        }
        self.registered_accounts.push(env::predecessor_account_id());
        self.get_mut_token_map(bet_token_id).insert(&env::predecessor_account_id(), &ini_balance);
    }
    
    pub fn get_current_prices(&self) -> Vec<(AccountId, Balance)> {
        self.current_prices.to_vec()
    }

    // Public method - accepts a greeting, such as "howdy", and records it
    pub fn set_current_price_for_token(&mut self, token: AccountId, price: Balance) {
        log!("Saving price for token {}: {}", token, price);
        self.current_prices.insert(&token, &price);          
    }

    fn get_mut_token_map(&mut self, bet_token_id: i8) -> &mut UnorderedMap<AccountId, Balance> {
        match bet_token_id {
            0 => {
                &mut self.heist_balances
            },
            1 => {
                &mut self.stheist_balances
            },
            _ => {
                panic!("Unknown bet_token_id {}", bet_token_id);
            }
        }
    }

    fn get_token_map(&self, bet_token_id: i8) -> &UnorderedMap<AccountId, Balance> {
        match bet_token_id {
            0 => {
                &self.heist_balances
            },
            1 => {
                &self.stheist_balances
            },
            _ => {
                panic!("Unknown bet_token_id {}", bet_token_id);
            }
        }
    }

    fn transfer(&mut self, from: &AccountId, to: &AccountId, bet_token_id: i8, amount: Balance) {
        let owner = self.owner.clone();
        let token_map = self.get_mut_token_map(bet_token_id);
        if from.as_str() != owner.as_str() {
            let balance = token_map.get(&from).unwrap_or(0);
            log!("Balance {}, amount {}", balance, amount);
            assert!(balance > amount, "Not enough balance");
            let new_balance = balance - amount;
            token_map.insert(&from, &new_balance);
        }
        if to.as_str() != owner.as_str() {
            let balance = token_map.get(&to).unwrap_or(0);
            let new_balance = balance + amount;
            token_map.insert(&to, &new_balance);
        }
    }

    /**
    * Modify so we only recieve the 
    */
    pub fn place_bet(&mut self, token: AccountId, bet_token_id: i8, amount: Balance) {
        let acc_id: AccountId = env::predecessor_account_id();
        self.transfer(&acc_id, &self.owner.clone(), bet_token_id, amount);
        // self.assert_sufficient_funds();

        let current_price = self.current_prices.get(&token).unwrap();
        let bet = Bet{
            token: token,
            initial_token_price: current_price,
            in_token: bet_token_id,
            amount: amount
        };

        
        log!("Acc id {}", acc_id);
        self.current_bets.insert(&acc_id, &bet);
    }

    pub fn get_bet_from_user(&self, account_id: AccountId) -> Bet {
        self.current_bets.get(&account_id).unwrap()
    }

    pub fn close_bet(&mut self) {
        let acc_id = env::predecessor_account_id();
        let bet = self.current_bets.get(&acc_id).unwrap();
        self.current_bets.remove(&acc_id);

        let current_price = self.current_prices.get(&bet.token).unwrap();

        let multiplicator = current_price * 1_000_000u128 / bet.initial_token_price / 1_000_000u128;
        let new_balance = bet.amount * multiplicator;

        self.transfer(&self.owner.clone(), &acc_id, bet.in_token, new_balance);
    }

    pub fn get_balance(&self, account_id: &AccountId, token_id: &i8) -> Balance {
        self.get_token_map(*token_id).get(account_id).unwrap()
    }

    pub fn get_stheist_price(&self) -> Balance {
        self.stheist_price
    }

    pub fn update_stheist_price(&mut self) {
        self.stheist_price = self.stheist_price * EPOCH_PRICE_MULTIPLIER / 1_000_000_000u128;
    }

    pub fn get_registered_accounts(&self) -> usize {
        self.registered_accounts.len()
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */

const OWNER: AccountId = "silkking.testnet".parse().unwrap();

#[cfg(test)]
mod tests {
    use super::*;

    

    #[test]
    fn register_with_heist() {
        let mut contract = PricePredictorContract::new(OWNER);

        contract.register(0);
        let balance = contract.get_balance(&"bob.near".parse().unwrap(), &0);
        
        assert!(balance == INITIAL_BALANCE, "Incorrect initial balance");

        let registered_accounts = contract.get_registered_accounts();
        assert!(registered_accounts == 1usize, "Incorrect registered accounts");
    }

    // #[test]
    // fn register_with_stheist() {
    //     let mut contract = PricePredictorContract::default();

    //     contract.register(1);
    //     let balance = contract.get_balance(&"bob.near".parse().unwrap(), &1);
    //     log!("register_with_stheist: Initial balance  {}. Var {}", balance, INITIAL_BALANCE);
    //     assert!(balance == INITIAL_BALANCE, "Incorrect initial balance");
    // }

    // #[test]
    // fn update_stheist_price_then() {
    //     let mut contract = PricePredictorContract::default();

    //     contract.update_stheist_price();
    //     contract.register(1);
    //     let balance = contract.get_balance(&"bob.near".parse().unwrap(), &1);
    //     log!("register_with_stheist: Initial balance  {}. Var {}", balance, INITIAL_BALANCE);
    //     assert!(balance < INITIAL_BALANCE, "Incorrect initial balance");
    // }

    // #[test]
    // fn set_then_get_price() {
    //     let mut contract = PricePredictorContract::default();
    //     contract.set_current_price_for_token("meta-pool.near".parse().unwrap(), 2500000000000000000000000);
    //     // let a = contract.get_current_prices();
    //     // print!("Saving price for token {:?}", a[0].1);
        
    // }

    // #[test]
    // fn insert_then_get_bet() {
    //     let mut contract = PricePredictorContract::default();
    //     contract.set_current_price_for_token("meta-pool.near".parse().unwrap(), 2500000000000000000000000);
    //     contract.register(0);
    //     contract.place_bet("meta-pool.near".parse().unwrap(), 0, 1);

    //     // let bet = contract.get_bet_from_user("bob.near".parse().unwrap());
    //     // log!("Bet {:?}", bet);
    // }

    // #[test]
    // fn insert_then_modify_price_then_get_balance() {
    //     let mut contract = PricePredictorContract::default();
    //     contract.set_current_price_for_token("meta-pool.near".parse().unwrap(), 1_000_000_000u128);
    //     contract.register(0);
    //     let initial_balance = contract.get_balance(&"bob.near".parse().unwrap(), &0);
    //     let bet_amount = 1_000u128;
    //     contract.place_bet("meta-pool.near".parse().unwrap(), 0, bet_amount);

    //     contract.set_current_price_for_token("meta-pool.near".parse().unwrap(), 2_000_000_000u128);
    //     contract.close_bet();

    //     let final_balance = contract.get_balance(&"bob.near".parse().unwrap(), &0);
        
    //     // log!("Initial balance {}, final balance {}", initial_balance, final_balance);
    //     assert!(final_balance - initial_balance == bet_amount, "Error rewards");
    // }

    // #[test]
    // fn update_then_get_stheist_price() {
    //     let mut contract = PricePredictorContract::default();

    //     let initial = contract.get_stheist_price();
    //     contract.update_stheist_price();
    //     let final_price = contract.get_stheist_price();

    //     // log!("Initial: {}. Final: {}", initial, final_price);
    //     assert!(final_price > initial, "Incorrect update");
    // }
}
