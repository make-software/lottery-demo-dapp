use core::cmp::min;
use odra::casper_types::U256;
use odra::casper_types::U512;
#[cfg(target_arch = "wasm32")]
use odra::odra_casper_wasm_env::casper_contract::contract_api::runtime;
use odra::prelude::*;
use odra::Address;
use odra::SubModule;
use odra::Var;
use odra_modules::access::Ownable;

/// Custom error type for the lottery contract
#[odra::odra_error]
pub enum Error {
    InsufficientPayment = 1,
    InvalidProbabilityConfiguration = 2,
}

/// Unique identifier for a lottery round
pub type RoundId = u32;

/// Unique identifier for a lottery round
pub type PlayId = U256;

/// Event emitted when a user enters the lottery
#[odra::event]
pub struct Play {
    /// Unique identifier for the player's entry
    pub play_id: PlayId,
    /// ID of the round the user entered
    pub round_id: RoundId,
    /// Address of the player
    pub player: Address,
    /// Timestamp of the entry
    pub timestamp: u64,
    /// Prize won by the player (if any)
    pub prize_amount: U512,
    /// Flag indicating if the entry won the jackpot
    pub is_jackpot: bool,
    /// Current jackpot amount after play
    pub jackpot_amount: U512,
}

/// Possible outcomes of a lottery play
enum Outcome {
    Jackpot,
    ConsolationPrize(U512),
    NoWin,
}

/// Main lottery contract struct
#[odra::module(events = [Play])]
pub struct Lottery {
    /// Ownable sub-module for managing contract ownership
    ownable: SubModule<Ownable>,
    /// Variable to store the ID of the current round
    current_round_id: Var<RoundId>,
    /// Variable to store the current play ID for the round
    current_play_id: Var<PlayId>,
    /// Variable to store collected fees
    collected_fees: Var<U512>,
    /// Variable to store the current prize pool
    prize_pool: Var<U512>,
    /// Variable to store the maximum consolation prize
    max_consolation_prize: Var<U512>,
    /// Variable to store the lottery fee
    lottery_fee: Var<U512>,
    /// Variable to store the jackpot winning probability
    jackpot_probability: Var<u8>,
    /// Variable to store the consolation prize winning probability
    consolation_prize_probability: Var<u8>,
    /// Variable to store the ticket price
    ticket_price: Var<U512>,
}

#[odra::module]
impl Lottery {
    /// Initializes the lottery contract with default configuration parameters
    pub fn init(
        &mut self,
        lottery_fee: U512,
        ticket_price: U512,
        max_consolation_prize: U512,
        jackpot_probability: u8,
        consolation_prize_probability: u8,
    ) {
        self.ownable.init();
        self.current_round_id.set(1);
        self.current_play_id.set(U256::one());
        self.collected_fees.set(U512::zero());
        self.prize_pool.set(U512::zero());
        self.lottery_fee.set(lottery_fee);
        self.ticket_price.set(ticket_price);
        self.max_consolation_prize.set(max_consolation_prize);
        self.assert_valid_probability_configuration(jackpot_probability);
        self.jackpot_probability.set(jackpot_probability);
        self.assert_valid_probability_configuration(consolation_prize_probability);
        self.consolation_prize_probability
            .set(consolation_prize_probability);
    }

    /// Functions from the submodules that are public in our smart contract
    delegate! {
        to self.ownable {
            fn get_owner(&self) -> Address;
            fn transfer_ownership(&mut self, new_owner: &Address);
        }
    }

    /*
    Admin
    */

    /// Configures various settings for the lottery contract.
    /// This function requires ownership.
    ///
    /// It would be correct for the changes in configuration to take effect starting
    /// the next round, but we will not do that to keep the implementation simpler
    pub fn configure(
        &mut self,
        ticket_price: Option<U512>,
        lottery_fee: Option<U512>,
        jackpot_probability: Option<u8>,
        max_consolation_prize: Option<U512>,
        consolation_prize_probability: Option<u8>,
    ) {
        self.ownable.assert_owner(&self.env().caller());

        if let Some(prize) = max_consolation_prize {
            self.max_consolation_prize.set(prize);
        }

        if let Some(fee) = lottery_fee {
            self.lottery_fee.set(fee);
        }

        if let Some(probability) = jackpot_probability {
            self.assert_valid_probability_configuration(probability);
            self.jackpot_probability.set(probability);
        }

        if let Some(probability) = consolation_prize_probability {
            self.assert_valid_probability_configuration(probability);
            self.consolation_prize_probability.set(probability);
        }

        if let Some(price) = ticket_price {
            self.ticket_price.set(price);
        }
    }

    /// Withdraws the collected fees
    /// This function requires ownership.
    pub fn transfer_fees_to_account(&mut self, amount: U512, receiver: Address) {
        self.ownable.assert_owner(&self.env().caller());
        self.collected_fees.subtract(amount);
        self.env().transfer_tokens(&receiver, &amount);
    }

    /// Adds the attached value to the prize pool.
    /// This function requires ownership.
    #[odra(payable)]
    pub fn top_up_prize_pool(&mut self) {
        self.ownable.assert_owner(&self.env().caller());
        self.prize_pool.add(self.env().attached_value());
    }

    /*
    Transactions
    */

    /// Allows a user to participate in the current lottery round.
    ///
    /// This function requires the lottery to be not paused, active, and a deposit
    /// to be attached to the message.
    ///
    /// It performs the following actions:
    ///  - Adds the attached value to the prize pool, deducting the lottery fee.
    ///  - Determines the outcome (jackpot, consolation prize, or no win).
    ///  - Distributes prizes and emits an event based on the outcome.
    #[odra(payable)]
    pub fn play_lottery(&mut self) {
        self.assert_payment_is_sufficient();
        let round_id = self.current_round_id.get_or_default();
        let play_id = self.current_play_id.get_or_default();
        let caller = self.env().caller();
        self.collected_fees.add(self.lottery_fee.get_or_default());
        self.prize_pool
            .add(self.env().attached_value() - self.lottery_fee.get_or_default());

        let mut is_jackpot = false;
        let mut prize_amount = U512::zero();

        match self.determine_outcome() {
            Outcome::Jackpot => {
                let prize_value = self.prize_pool.get_or_default();
                self.env().transfer_tokens(&caller, &prize_value);
                self.prize_pool.set(U512::zero());

                // update play event state
                is_jackpot = true;
                prize_amount = prize_value;

                // start the next round
                self.current_round_id.add(1);
            }
            Outcome::ConsolationPrize(prize_value) => {
                self.env().transfer_tokens(&caller, &prize_value);
                self.prize_pool.subtract(prize_value);

                // update play event state
                prize_amount = prize_value;
            }
            _ => {}
        }

        self.current_play_id.add(U256::one());
        self.env().emit_event(Play {
            round_id,
            player: caller,
            play_id,
            timestamp: self.env().get_block_time(),
            is_jackpot,
            prize_amount,
            jackpot_amount: self.prize_pool.get_or_default(),
        });
    }

    /*
    Queries
    */

    /// Returns the current prize pool of the lottery.
    pub fn prize_pool(&self) -> U512 {
        self.prize_pool.get_or_default()
    }

    /// Returns the ticket price for the lottery.
    pub fn ticket_price(&self) -> Option<U512> {
        self.ticket_price.get()
    }

    /*
    Internal
    */

    /// Determines lottery outcome based on random number and probabilities.
    /// Uses secure random number for WASM32 target.
    fn determine_outcome(&self) -> Outcome {
        let random_number = 4; // Used fair dice, so it is random
        #[cfg(target_arch = "wasm32")]
        let random_number = self.get_random_number(); // if in wasm, get "real", "random" number
        let total_probability = self.jackpot_probability.get_or_default()
            + self.consolation_prize_probability.get_or_default();

        if random_number <= self.jackpot_probability.get_or_default().into() {
            Outcome::Jackpot
        } else if random_number <= total_probability.into() {
            let max_consolation_prize = min(
                self.max_consolation_prize.get_or_default(),
                self.prize_pool.get_or_default(),
            )
            .as_u128();
            let scaling_factor = max_consolation_prize / 100;
            let consolation_prize =
                U512::from((scaling_factor * random_number as u128) % max_consolation_prize);
            Outcome::ConsolationPrize(consolation_prize)
        } else {
            Outcome::NoWin
        }
    }

    /// Generates cryptographically secure random number (WASM32 only).
    #[cfg(target_arch = "wasm32")]
    fn get_random_number(&self) -> u64 {
        let random_bytes: [u8; 32] = runtime::random_bytes();
        u64::from_be_bytes(random_bytes[..8].try_into().unwrap()) % 100
    }

    /// Ensures attached value matches ticket price. Reverts if wrong.
    fn assert_payment_is_sufficient(&self) {
        if self.env().attached_value() < self.ticket_price.get_or_default() {
            self.env().revert(Error::InsufficientPayment);
        }
    }

    /// Ensures probability value is within valid range (0-100). Reverts if invalid.
    fn assert_valid_probability_configuration(&self, value: u8) {
        if value > 100 {
            self.env().revert(Error::InvalidProbabilityConfiguration)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lottery::{Error, LotteryHostRef, LotteryInitArgs, Play};
    use core::ops::Add;
    use odra::{
        casper_types::{U256, U512},
        host::{Deployer, HostRef},
    };

    const ONE_HOUR_IN_MILLISECONDS: u64 = 3_600_000;
    const ONE_CSPR_IN_MOTES: u64 = 1_000_000_000;

    #[test]
    fn basic_flow() {
        let consolation = (ONE_CSPR_IN_MOTES * 49 / 100 * 4) % (ONE_CSPR_IN_MOTES * 49);
        print!("consolation: {:?}", consolation);
        let env = odra_test::env();
        let admin = env.get_account(0);
        let alice = env.get_account(1);
        let bob = env.get_account(2);
        let charlie = env.get_account(3);

        env.advance_block_time(ONE_HOUR_IN_MILLISECONDS);

        let mut expected_round = 1;
        let expected_play: U256 = U256::from(1);

        env.set_caller(admin);
        let mut contract = LotteryHostRef::deploy(
            &env,
            LotteryInitArgs {
                lottery_fee: U512::from(1 * ONE_CSPR_IN_MOTES),
                ticket_price: U512::from(50 * ONE_CSPR_IN_MOTES),
                max_consolation_prize: U512::from(50 * ONE_CSPR_IN_MOTES),
                jackpot_probability: 100,
                consolation_prize_probability: 40,
            },
        );
        env.set_caller(alice);

        assert_eq!(
            contract
                .with_tokens(U512::from(1 * ONE_CSPR_IN_MOTES))
                .try_play_lottery(),
            Err(Error::InsufficientPayment.into())
        );

        contract
            .with_tokens(U512::from(50 * ONE_CSPR_IN_MOTES))
            .play_lottery();

        assert!(env.emitted_event(
            contract.address(),
            &Play {
                round_id: expected_round,
                player: alice,
                play_id: expected_play,
                prize_amount: U512::from(49 * ONE_CSPR_IN_MOTES),
                is_jackpot: true,
                timestamp: ONE_HOUR_IN_MILLISECONDS,
                jackpot_amount: U512::zero(),
            },
        ));
        assert_eq!(env.events_count(contract.address()), 2);
        expected_round += 1; // expected next round
        let expected_play = expected_play.add(1); // expected next play

        env.set_caller(bob);
        contract
            .with_tokens(U512::from(50 * ONE_CSPR_IN_MOTES))
            .play_lottery();

        assert!(env.emitted_event(
            contract.address(),
            &Play {
                round_id: expected_round,
                player: bob,
                play_id: expected_play,
                prize_amount: U512::from(49 * ONE_CSPR_IN_MOTES),
                is_jackpot: true,
                timestamp: ONE_HOUR_IN_MILLISECONDS,
                jackpot_amount: U512::zero(),
            },
        ));
        assert_eq!(env.events_count(contract.address()), 3);
        expected_round += 1; // expected next round
        let expected_play = expected_play.add(1); // expected next play

        env.set_caller(charlie);
        contract
            .with_tokens(U512::from(50 * ONE_CSPR_IN_MOTES))
            .play_lottery();

        assert!(env.emitted_event(
            contract.address(),
            &Play {
                round_id: expected_round,
                player: charlie,
                play_id: expected_play,
                prize_amount: U512::from(49 * ONE_CSPR_IN_MOTES),
                is_jackpot: true,
                timestamp: ONE_HOUR_IN_MILLISECONDS,
                jackpot_amount: U512::zero(),
            },
        ));
        assert_eq!(env.events_count(contract.address()), 4);
        expected_round += 1;
        let expected_play = expected_play.add(1);

        // ---------------------------------------------------------------------
        // configure contract with no possibility to win
        // ---------------------------------------------------------------------

        env.set_caller(admin);
        contract.configure(
            Some(U512::from(50 * ONE_CSPR_IN_MOTES)), // ticket_price
            Some(U512::from(1 * ONE_CSPR_IN_MOTES)),  // lottery_fee
            Some(0),                                  // jackpot_probability
            Some(U512::from(50 * ONE_CSPR_IN_MOTES)), // max_consolation_prize
            Some(0),                                  // consolation_prize_probability
        );

        env.set_caller(charlie);
        contract
            .with_tokens(U512::from(50 * ONE_CSPR_IN_MOTES))
            .play_lottery();

        assert!(env.emitted_event(
            contract.address(),
            &Play {
                round_id: expected_round,
                player: charlie,
                play_id: expected_play,
                prize_amount: U512::zero(),
                is_jackpot: false,
                timestamp: ONE_HOUR_IN_MILLISECONDS,
                jackpot_amount: U512::from(49 * ONE_CSPR_IN_MOTES),
            },
        ));
        assert_eq!(env.events_count(contract.address()), 5);
        let expected_play = expected_play.add(1);

        // ---------------------------------------------------------------------
        // configure contract with possibility to win only consolation_prize
        // ---------------------------------------------------------------------

        env.set_caller(admin);
        contract.configure(
            Some(U512::from(50 * ONE_CSPR_IN_MOTES)), // ticket_price
            Some(U512::from(1 * ONE_CSPR_IN_MOTES)),  // lottery_fee
            Some(0),                                  // jackpot_probability
            Some(U512::from(50 * ONE_CSPR_IN_MOTES)), // max_consolation_prize
            Some(100),                                // consolation_prize_probability
        );

        env.set_caller(charlie);
        contract
            .with_tokens(U512::from(50 * ONE_CSPR_IN_MOTES))
            .play_lottery();

        assert!(env.emitted_event(
            contract.address(),
            &Play {
                round_id: expected_round,
                player: charlie,
                play_id: expected_play,
                prize_amount: U512::from(2 * ONE_CSPR_IN_MOTES),
                is_jackpot: false,
                timestamp: ONE_HOUR_IN_MILLISECONDS,
                jackpot_amount: U512::from(96 * ONE_CSPR_IN_MOTES),
            },
        ));
        assert_eq!(env.events_count(contract.address()), 6);
    }
}
