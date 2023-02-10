// Find all our documentation at https://docs.near.org
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, log, near_bindgen, Balance, AccountId};

mod internal;

const EPOCH_PRICE_MULTIPLIER: u128 = 1000125079u128;
const INITIAL_BALANCE: u128 = 100_000_000_000_000_000_000_000_000;
const WORD_PRICE: u128 = INITIAL_BALANCE;


#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Bet {
    token: AccountId,
    initial_token_price: Balance,
    in_token: i8,
    amount: Balance,
    long: bool,
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
    words: Vec<String>,
    users_word_amount: UnorderedMap<AccountId, usize>,
}

// Define the default, which automatically initializes the contract
impl Default for PricePredictorContract {
    fn default() -> Self{
        Self{
            owner: "silkking.testnet".parse().unwrap(),
            registered_accounts: Vec::new(),
            current_prices: UnorderedMap::new(b"current_prices".to_vec()),
            current_bets: UnorderedMap::new(b"current_bets".to_vec()),
            heist_balances: UnorderedMap::new(b"heist_balances".to_vec()),
            stheist_balances: UnorderedMap::new(b"stheist_balances".to_vec()),
            stheist_price: 1_000_000_000_000u128,
            words: vec![
                "word1".to_string(), 
                "word2".to_string(), 
                "word3".to_string(), 
                "word4".to_string(), 
                "word5".to_string(), 
                "word6".to_string(), 
                "word7".to_string(), 
                "word8".to_string(), 
                "word9".to_string(), 
                "word10".to_string(), 
                "word11".to_string(), 
                "word12".to_string()
            ],
            users_word_amount: UnorderedMap::new(b"users_word_amount".to_vec()),
        }
    }
}


// Implement the contract structure
#[near_bindgen]
impl PricePredictorContract {

    // #[init]
    // pub fn new(owner_id: AccountId) -> Self {
    //     Self {
    //         owner: owner_id,
    //         registered_accounts: Vec::new(),
    //         current_prices: UnorderedMap::new(b"current_prices".to_vec()),
    //         current_bets: UnorderedMap::new(b"current_bets".to_vec()),
    //         heist_balances: UnorderedMap::new(b"heist_balances".to_vec()),
    //         stheist_balances: UnorderedMap::new(b"stheist_balances".to_vec()),
    //         stheist_price: 1_000_000_000_000u128,
    //     }
    // }

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
        self.assert_owner();
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
            log!("123456789 {} {}", balance, amount);
            token_map.insert(&to, &new_balance);
        }
    }

    /**
    * Modify so we only recieve the 
    */
    pub fn place_bet(&mut self, token: AccountId, bet_token_id: i8, amount: Balance, long: bool) {
        let acc_id: AccountId = env::predecessor_account_id();
        self.transfer(&acc_id, &self.owner.clone(), bet_token_id, amount);
        // self.assert_sufficient_funds();

        let current_price = self.current_prices.get(&token).unwrap();
        let bet = Bet{
            token,
            initial_token_price: current_price,
            in_token: bet_token_id,
            amount: amount,
            long
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

        let multiplicator;
        if bet.long {
            multiplicator = current_price * 1_000_000u128 / bet.initial_token_price / 1_000_000u128;
        } else {
            multiplicator = bet.initial_token_price * 1_000_000u128 / current_price / 1_000_000u128;
        }
        
        let new_balance = bet.amount * multiplicator;
        log!("New balance {}", new_balance);

        self.transfer(&self.owner.clone(), &acc_id, bet.in_token, new_balance);
    }

    pub fn get_balance(&self, account_id: &AccountId, token_id: &i8) -> Balance {
        self.get_token_map(*token_id).get(account_id).unwrap()
    }

    pub fn get_stheist_price(&self) -> Balance {
        self.stheist_price
    }

    pub fn update_stheist_price(&mut self) {
        self.assert_owner();
        self.stheist_price = self.stheist_price * EPOCH_PRICE_MULTIPLIER / 1_000_000_000u128;
    }

    pub fn get_registered_accounts(&self) -> usize {
        self.registered_accounts.len()
    }

    pub fn get_user_word_amount(&self, account_id: AccountId) -> usize {
        self.users_word_amount.get(&account_id).unwrap_or(0usize)
    }

    pub fn get_user_words(&mut self) -> Vec<String> {
        let user_word_amount = self.users_word_amount.get(&env::predecessor_account_id()).unwrap_or(0usize);
        self.words[0..user_word_amount].to_vec()
    }

    pub fn get_word_price(&mut self) -> (Balance, Balance) {
        let stheist_word_price = WORD_PRICE * 1_000_000u128 / self.stheist_price / 1_000_000u128;
        (WORD_PRICE, stheist_word_price)
    }

    pub fn buy_word(&mut self, token_id: i8) -> String {
        let word_prices = self.get_word_price();
        let price = if token_id == 0 { word_prices.0 } else { word_prices.1 };
        let user_balance = self.get_balance(&env::predecessor_account_id(), &token_id);
        assert!(user_balance >= price, "Not enough tokens");

        self.transfer(&env::predecessor_account_id(), &self.owner.clone(), token_id, price);
        let curr_word = self.get_user_word_amount(env::predecessor_account_id());
        self.users_word_amount.insert(&env::predecessor_account_id(), &(curr_word + 1));
        self.words[curr_word].clone()
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context_owner(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id("silkking.testnet".parse().unwrap())
            .is_view(is_view)
            .build()
    }

    fn get_context_normal(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id("bob.testnet".parse().unwrap())
            .is_view(is_view)
            .build()
    }
    
    #[test]
    fn register_with_heist() {
        let context = get_context_normal(false);
        testing_env!(context.clone());

        // let mut contract = PricePredictorContract::new(OWNER);
        let mut contract = PricePredictorContract::default();

        contract.register(0);
        let balance = contract.get_balance(&context.predecessor_account_id, &0);
        
        assert!(balance == INITIAL_BALANCE, "Incorrect initial balance");

        let registered_accounts = contract.get_registered_accounts();
        assert!(registered_accounts == 1usize, "Incorrect registered accounts");
    }

    #[test]
    fn register_with_stheist() {
        let context = get_context_normal(false);
        testing_env!(context.clone());

        let mut contract = PricePredictorContract::default();

        contract.register(1);
        let balance = contract.get_balance(&context.predecessor_account_id, &1);
        // log!("register_with_stheist: Initial balance  {}. Var {}", balance, INITIAL_BALANCE);
        assert!(balance == INITIAL_BALANCE, "Incorrect initial balance");
    }

    #[test]
    fn update_stheist_price_then() {
        let context = get_context_owner(false);
        testing_env!(context.clone());
        let mut contract = PricePredictorContract::default();

        contract.update_stheist_price();
        contract.register(1);
        let balance = contract.get_balance(&context.predecessor_account_id, &1);
        // log!("register_with_stheist: Initial balance  {}. Var {}", balance, INITIAL_BALANCE);
        assert!(balance < INITIAL_BALANCE, "Incorrect initial balance");
    }

    #[test]
    fn set_then_get_price() {
        let context = get_context_owner(false);
        testing_env!(context);

        let mut contract = PricePredictorContract::default();
        contract.set_current_price_for_token("meta-pool.near".parse().unwrap(), 2500000000000000000000000);
        // let a = contract.get_current_prices();
        // print!("Saving price for token {:?}", a[0].1);
        
    }

    #[test]
    fn insert_then_get_bet() {
        let context_owner = get_context_owner(false);
        let context_normal = get_context_normal(false);
        testing_env!(context_owner.clone());

        let mut contract = PricePredictorContract::default();
        contract.set_current_price_for_token("meta-pool.near".parse().unwrap(), 2500000000000000000000000);

        testing_env!(context_normal.clone());
        contract.register(0);
        contract.place_bet("meta-pool.near".parse().unwrap(), 0, 1, true);

        // let bet = contract.get_bet_from_user("bob.near".parse().unwrap());
        // log!("Bet {:?}", bet);
    }

    #[test]
    fn insert_then_modify_price_then_get_balance() {
        let context_owner = get_context_owner(false);
        let context_normal = get_context_normal(false);

        testing_env!(context_owner.clone());
        let mut contract = PricePredictorContract::default();
        contract.set_current_price_for_token("meta-pool.near".parse().unwrap(), 1_000_000_000u128);

        testing_env!(context_normal.clone());
        contract.register(0);
        
        let initial_balance = contract.get_balance(&context_normal.predecessor_account_id, &0);
        let bet_amount = 1_000u128;
        contract.place_bet("meta-pool.near".parse().unwrap(), 0, bet_amount, true);

        testing_env!(context_owner.clone());
        contract.set_current_price_for_token("meta-pool.near".parse().unwrap(), 2_000_000_000u128);

        testing_env!(context_normal.clone());
        contract.close_bet();

        let final_balance = contract.get_balance(&context_normal.predecessor_account_id, &0);
        
        assert!(final_balance - initial_balance == bet_amount, "Error rewards");
    }

    #[test]
    fn update_then_get_stheist_price() {
        let context = get_context_owner(false);
        testing_env!(context);

        let mut contract = PricePredictorContract::default();

        let initial = contract.get_stheist_price();
        contract.update_stheist_price();
        let final_price = contract.get_stheist_price();

        // log!("Initial: {}. Final: {}", initial, final_price);
        assert!(final_price > initial, "Incorrect update");
    }

    #[test]
    fn get_buy_get_word() {
        let context = get_context_normal(false);
        testing_env!(context.clone());

        let mut contract = PricePredictorContract::default();

        let word_amount_1 = contract.get_user_word_amount(context.clone().predecessor_account_id);
        assert!(word_amount_1 == 0, "Incorrect word amount");

        let words = contract.get_user_words();
        assert!(do_vecs_match(&words, &Vec::<String>::new()), "Error words");

        contract.register(1);
        contract.buy_word(1);

        let word_amount_2 = contract.get_user_word_amount(context.clone().predecessor_account_id);
        assert!(word_amount_2 == 1, "Incorrect word amount");

        let words_2 = contract.get_user_words();
        assert!(do_vecs_match(&words_2, &vec!["word1".to_string()]), "Error words");
        

    }

    fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }
}
