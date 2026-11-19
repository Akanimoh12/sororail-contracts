// ------------------------------------------------------------------ init

use super::*;
use soroban_sdk::{testutils::Events as _, Event as _};

#[test]
fn init_stores_the_agreement() {
    let f = Fixture::new(true);
    let e = f.client.get();
    assert_eq!(e.depositor, f.depositor);
    assert_eq!(e.beneficiary, f.beneficiary);
    assert_eq!(e.arbiter, Some(f.arbiter.clone()));
    assert_eq!(e.amount, AMOUNT);
    assert_eq!(e.deadline, DEADLINE);
    assert_eq!(e.state, State::Created);
    // Nothing has moved yet.
    assert_eq!(f.escrow_balance(), 0);
}

#[test]
fn init_rejects_a_second_call() {
    let f = Fixture::new(true);
    let res = f.client.try_init(
        &f.depositor,
        &f.beneficiary,
        &None,
        &f.token.address,
        &AMOUNT,
        &DEADLINE,
    );
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn init_rejects_non_positive_amounts() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = START_TS);
    let client = EscrowContractClient::new(&env, &env.register(EscrowContract, ()));
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let t = Address::generate(&env);

    for bad in [0i128, -1, i128::MIN] {
        let res = client.try_init(&a, &b, &None, &t, &bad, &DEADLINE);
        assert_eq!(
            res,
            Err(Ok(Error::InvalidAmount)),
            "amount {bad} was accepted"
        );
    }
}

#[test]
fn init_rejects_a_deadline_that_is_not_in_the_future() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = START_TS);
    let client = EscrowContractClient::new(&env, &env.register(EscrowContract, ()));
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let t = Address::generate(&env);

    // Equal to now, and in the past.
    for bad in [START_TS, START_TS - 1, 0] {
        let res = client.try_init(&a, &b, &None, &t, &AMOUNT, &bad);
        assert_eq!(
            res,
            Err(Ok(Error::InvalidTimeRange)),
            "deadline {bad} was accepted"
        );
    }
}

#[test]
fn entry_points_error_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let client = EscrowContractClient::new(&env, &env.register(EscrowContract, ()));
    let who = Address::generate(&env);

    assert_eq!(client.try_fund(), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_release(&who), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_refund(&who), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_dispute(&who), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_resolve(&5_000), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_state(), Err(Ok(Error::NotInitialized)));
}

#[test]
#[should_panic]
fn init_requires_the_depositors_authorization() {
    // No mock_all_auths: require_auth must actually bite.
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = START_TS);
    let client = EscrowContractClient::new(&env, &env.register(EscrowContract, ()));
    client.init(
        &Address::generate(&env),
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &AMOUNT,
        &DEADLINE,
    );
}

#[test]
fn init_emits_created_event_with_correct_topics_and_data() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = START_TS);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let issuer = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(issuer).address();
    let client = EscrowContractClient::new(&env, &env.register(EscrowContract, ()));

    client.init(
        &depositor,
        &beneficiary,
        &Some(arbiter.clone()),
        &token_address,
        &AMOUNT,
        &DEADLINE,
    );

    let all = env.events().all().filter_by_contract(&client.address);
    let events = all.events();
    assert_eq!(events.len(), 1);

    let expected = Created {
        depositor,
        beneficiary,
        token: token_address,
        amount: AMOUNT,
        deadline: DEADLINE,
    };
    assert_eq!(&events[0], &expected.to_xdr(&env, &client.address));
}
