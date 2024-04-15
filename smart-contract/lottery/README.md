
# Lottery Smart Contract - Demo

This smart contract allows users to participate in a demo lottery on the Casper blockchain. Users can purchase tickets and have a chance to win the prize pool, which is accumulated from ticket purchases or a consolation prize.

## Functionalities

* Configure settings like ticket price, maximum consolation prize, prize probabilities, and fees.
* Allow users to participate in the lottery.
* Determine lottery winners based on a pseudo-random number generation mechanism (for demo purposes only).
* Distribute jackpot and consolation prizes to winners.
* Provide owner with administrative functionalities topping up the prize pool, and configuring settings.

## Entry Points (Public Functions)

### Admin
These functionalities can only be called by the contract owner (designated during deployment). They are used for configuration and maintenance purposes.

  * `configure(max_consolation_prize: Option<U512>, lottery_fee: Option<U512>, jackpot_probability: Option<u8>, consolation_prize_probability: Option<u8>, ticket_price: Option<U512>)`: Configures various lottery settings.
  * `top_up_prize_pool()`: Allows the owner to add funds to the prize pool.
  * `transfer_fees_to_account(amount: U512, reciver: Address)`: Transfers the requested amount to receiver`s address. Reverts if the requested amount bigger than collected fees.

### User

  * `play_lottery()`: Participates in the current lottery round by purchasing a ticket (requires sending the ticket price in attached value). This is a payable entry point that needs to be called using `proxy_caller`.

## Queries (Public View Functions)

* `ticket_price()`: Returns the current ticket price.
* `prize_pool()`: Returns the current prize pool balance.

## Interaction Example

1. The contract owner deploys the contract and creates the first lottery round.
2. Users can then check the ticket price using `ticket_price()`.
3. To participate, a user calls `play_lottery()` while attaching the required amount (ticket price) in Casper tokens (CSPR).
4. **Note:** Whether the user is a winner is immediately determined based on a pseudo-random number generation mechanism (for demo purposes only). This mechanism is not suitable for production use.
5. Winners receive their prizes automatically (jackpot or consolation prize) based on the outcome.

## Deployment Addresses

**- Testnet:**

  - contract hash: `hash-40777e199af2ae4756c2a148c24e79885dc062fe4428adf23212dd04fd73187b`

**- Mainnet (if applicable):**

  - contract hash: coming soon.

## Security Considerations

- This smart contract is a demo application not ready for production use. It uses a naive pseudo-random number generation mechanism. A secure random number generation mechanism is crucial for real-world lottery applications.


## Usage

It's recommended to install `cargo-odra` first. You can find it here: [https://github.com/odradev/odra](https://github.com/odradev/odra)

### Build

```
make build
```

### Test

To run tests on your local machine, you can simply execute the following command:

```
make test
```

To test actual WASM files against a backend

```
make test-wasm
```
