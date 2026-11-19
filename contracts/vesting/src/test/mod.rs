// Test fixtures do plain arithmetic on known-small constants; the checked-math
// rule is for contract code.
#![allow(clippy::arithmetic_side_effects)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::TokenClient,
    Address, Env,
};
use sororail_common::{testutils::TestEnv, Error};

use crate::{
    contract::{VestingContract, VestingContractClient},
    events::{Claimed, Revoked},
    types::Grant,
};

mod claim;
mod conservation;
mod create;
mod revoke;
mod schedule;

const TOTAL: i128 = 1_000_000;
const START: u64 = 1_000_000;
const CLIFF: u64 = 100;
const DURATION: u64 = 1_000;
const MINT: i128 = TOTAL * 10;

struct Fixture<'a> {
    env: Env,
    client: VestingContractClient<'a>,
    token: TokenClient<'a>,
    grantor: Address,
    beneficiary: Address,
}

impl Fixture<'_> {
    fn new(revocable: bool) -> Self {
        Fixture::with_schedule(revocable, CLIFF, DURATION)
    }

    fn with_schedule(revocable: bool, cliff: u64, duration: u64) -> Self {
        let te = TestEnv::at(START);
        let (token, grantor) = te.make_token(MINT);
        let token_address = token.address.clone();
        let beneficiary = te.make_address();
        let env = te.env;

        let client = VestingContractClient::new(&env, &env.register(VestingContract, ()));
        client.create(
            &grantor,
            &beneficiary,
            &token_address,
            &TOTAL,
            &START,
            &cliff,
            &duration,
            &revocable,
        );

        Fixture {
            token: TokenClient::new(&env, &token_address),
            env,
            client,
            grantor,
            beneficiary,
        }
    }

    fn at(&self, ts: u64) {
        self.env.ledger().with_mut(|l| l.timestamp = ts);
    }

    fn held(&self) -> i128 {
        self.token.balance(&self.client.address)
    }

    fn assert_conserved(&self) {
        let g = self.client.get();
        assert_eq!(
            g.claimed + g.returned + self.client.remaining(),
            g.total,
            "conservation violated: {g:?}"
        );
        assert_eq!(
            self.held(),
            self.client.remaining(),
            "held balance diverged from remaining: {g:?}"
        );
    }
}

/// A bare grant for testing the schedule in isolation.
fn bare(env: &Env, cliff: u64, duration: u64) -> Grant {
    Grant {
        grantor: Address::generate(env),
        beneficiary: Address::generate(env),
        token: Address::generate(env),
        total: TOTAL,
        start: START,
        cliff,
        duration,
        revocable: true,
        claimed: 0,
        returned: 0,
        revoked_at: None,
    }
}
