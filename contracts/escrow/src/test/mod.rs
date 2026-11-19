use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    Address, Env,
};
use sororail_common::Error;

use crate::{
    contract::{EscrowContract, EscrowContractClient},
    events::{Created, Disputed, Funded, Refunded, Released, Resolved},
    types::State,
};

mod dispute;
mod fund;
mod init;
mod refund;
mod release;
mod resolve;

const AMOUNT: i128 = 1_000_000;
const START_TS: u64 = 1_000_000;
const DEADLINE: u64 = START_TS + 10_000;

struct Fixture<'a> {
    env: Env,
    client: EscrowContractClient<'a>,
    token: TokenClient<'a>,
    depositor: Address,
    beneficiary: Address,
    arbiter: Address,
    outsider: Address,
}

impl Fixture<'_> {
    /// Builds an initialized escrow. `with_arbiter` controls whether the
    /// dispute path is available.
    fn new(with_arbiter: bool) -> Self {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = START_TS);

        let depositor = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let arbiter = Address::generate(&env);
        let outsider = Address::generate(&env);

        let issuer = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(issuer);
        let token_address = sac.address();
        StellarAssetClient::new(&env, &token_address).mint(&depositor, &(AMOUNT * 10));

        let contract_id = env.register(EscrowContract, ());
        let client = EscrowContractClient::new(&env, &contract_id);

        client.init(
            &depositor,
            &beneficiary,
            &if with_arbiter {
                Some(arbiter.clone())
            } else {
                None
            },
            &token_address,
            &AMOUNT,
            &DEADLINE,
        );

        Fixture {
            token: TokenClient::new(&env, &token_address),
            env,
            client,
            depositor,
            beneficiary,
            arbiter,
            outsider,
        }
    }

    fn funded(with_arbiter: bool) -> Self {
        let f = Fixture::new(with_arbiter);
        f.client.fund();
        f
    }

    fn escrow_balance(&self) -> i128 {
        self.token.balance(&self.client.address)
    }

    fn advance_past_deadline(&self) {
        self.env.ledger().with_mut(|l| l.timestamp = DEADLINE + 1);
    }
}
