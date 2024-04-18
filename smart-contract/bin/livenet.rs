use lottery::lottery::{LotteryHostRef, LotteryInitArgs};
use odra::casper_types::U512;
use odra::host::Deployer;

const ONE_CSPR_IN_MOTES: u64 = 1_000_000_000;

fn main() {
    let env = odra_casper_livenet_env::env();

    let init_args = LotteryInitArgs {
        lottery_fee: U512::from(1 * ONE_CSPR_IN_MOTES),
        ticket_price: U512::from(50 * ONE_CSPR_IN_MOTES),
        max_consolation_prize: U512::from(50 * ONE_CSPR_IN_MOTES),
        jackpot_probability: 1,
        consolation_prize_probability: 10,
    };

    env.set_gas(200_000_000_000u64);
    LotteryHostRef::deploy(&env, init_args);
    println!("Success!");
}
