// ------------------------------------------------------------------ create

use super::*;
use soroban_sdk::{map, testutils::Events as _, vec, IntoVal, Symbol, Val};

#[test]
fn create_funds_the_whole_grant_up_front() {
    let f = Fixture::new(true);
    assert_eq!(f.held(), TOTAL);
    assert_eq!(f.token.balance(&f.grantor), MINT - TOTAL);
    f.assert_conserved();
}

#[test]
fn create_rejects_a_non_positive_total() {
    let env = Env::default();
    env.mock_all_auths();
    let c = VestingContractClient::new(&env, &env.register(VestingContract, ()));
    let (a, b, t) = (
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    );
    for bad in [0i128, -1, i128::MIN] {
        assert_eq!(
            c.try_create(&a, &b, &t, &bad, &START, &CLIFF, &DURATION, &true),
            Err(Ok(Error::InvalidAmount)),
            "total {bad} was accepted"
        );
    }
}

#[test]
fn create_rejects_a_zero_duration() {
    let env = Env::default();
    env.mock_all_auths();
    let c = VestingContractClient::new(&env, &env.register(VestingContract, ()));
    let (a, b, t) = (
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    );
    assert_eq!(
        c.try_create(&a, &b, &t, &TOTAL, &START, &0, &0, &true),
        Err(Ok(Error::InvalidDuration))
    );
}

#[test]
fn create_rejects_a_cliff_after_the_end() {
    let env = Env::default();
    env.mock_all_auths();
    let c = VestingContractClient::new(&env, &env.register(VestingContract, ()));
    let (a, b, t) = (
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    );
    assert_eq!(
        c.try_create(
            &a,
            &b,
            &t,
            &TOTAL,
            &START,
            &(DURATION + 1),
            &DURATION,
            &true
        ),
        Err(Ok(Error::VestingCliffAfterEnd))
    );
}

/// When several create inputs are invalid at once, the first check in
/// `create`'s validation sequence wins. Locks the order:
/// `require_positive(total)` → `duration == 0` → `cliff > duration`.
#[test]
fn create_reports_the_first_validation_error_when_several_apply() {
    let env = Env::default();
    env.mock_all_auths();
    let c = VestingContractClient::new(&env, &env.register(VestingContract, ()));
    let (a, b, t) = (
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    );

    // total=-1, duration=0, cliff=5: InvalidAmount beats InvalidDuration /
    // VestingCliffAfterEnd.
    assert_eq!(
        c.try_create(&a, &b, &t, &-1, &START, &5, &0, &true),
        Err(Ok(Error::InvalidAmount))
    );

    // total ok, duration=0, cliff=5: InvalidDuration beats VestingCliffAfterEnd.
    assert_eq!(
        c.try_create(&a, &b, &t, &TOTAL, &START, &5, &0, &true),
        Err(Ok(Error::InvalidDuration))
    );
}

#[test]
fn create_rejects_a_second_call() {
    let f = Fixture::new(true);
    assert_eq!(
        f.client.try_create(
            &f.grantor,
            &f.beneficiary,
            &f.token.address,
            &TOTAL,
            &START,
            &CLIFF,
            &DURATION,
            &true
        ),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn create_rejects_identical_grantor_and_beneficiary() {
    let env = Env::default();
    env.mock_all_auths();
    let c = VestingContractClient::new(&env, &env.register(VestingContract, ()));
    let party = Address::generate(&env);
    let token = Address::generate(&env);
    assert_eq!(
        c.try_create(&party, &party, &token, &TOTAL, &START, &CLIFF, &DURATION, &true),
        Err(Ok(Error::IdenticalParties))
    );
}

/// `create` persists the grant before the token pull. A failed transfer must
/// roll back that write so `get` still returns `NotInitialized`.
#[test]
fn create_reverts_cleanly_when_grantor_cannot_fund() {
    let te = TestEnv::at(START);
    let (token_client, grantor) = te.make_token(1);
    let token = token_client.address.clone();
    let beneficiary = te.make_address();
    let env = te.env.clone();
    let c = VestingContractClient::new(&env, &env.register(VestingContract, ()));

    assert!(c
        .try_create(
            &grantor,
            &beneficiary,
            &token,
            &TOTAL,
            &START,
            &CLIFF,
            &DURATION,
            &true
        )
        .is_err());
    assert_eq!(c.try_get(), Err(Ok(Error::NotInitialized)));
    assert_eq!(token_client.balance(&c.address), 0);
    assert_eq!(token_client.balance(&grantor), 1);
}

/// Pins the wire shape indexers decode (SPEC.md `indexed_events`). The
/// expected value is spelled out literally rather than built from
/// `events::Created`, so renaming a topic or a field fails here.
#[test]
fn create_emits_the_created_event() {
    let te = TestEnv::at(START);
    let (token_client, grantor) = te.make_token(MINT);
    let token_address = token_client.address.clone();
    let beneficiary = te.make_address();
    let env = te.env;

    let client = VestingContractClient::new(&env, &env.register(VestingContract, ()));
    client.create(
        &grantor,
        &beneficiary,
        &token_address,
        &TOTAL,
        &START,
        &CLIFF,
        &DURATION,
        &true,
    );

    let total: Val = TOTAL.into_val(&env);
    let start: Val = START.into_val(&env);
    let cliff: Val = CLIFF.into_val(&env);
    let duration: Val = DURATION.into_val(&env);
    let revocable: Val = true.into_val(&env);
    // Map data: keys are the field names, serialized in sorted order.
    let data: Val = map![
        &env,
        (Symbol::new(&env, "beneficiary"), beneficiary.into_val(&env)),
        (Symbol::new(&env, "cliff"), cliff),
        (Symbol::new(&env, "duration"), duration),
        (Symbol::new(&env, "revocable"), revocable),
        (Symbol::new(&env, "start"), start),
        (Symbol::new(&env, "token"), token_address.into_val(&env)),
        (Symbol::new(&env, "total"), total),
    ]
    .into_val(&env);
    assert_eq!(
        env.events().all().filter_by_contract(&client.address),
        vec![
            &env,
            (
                client.address.clone(),
                vec![
                    &env,
                    Symbol::new(&env, "vesting").into_val(&env),
                    Symbol::new(&env, "created").into_val(&env),
                    grantor.into_val(&env),
                ],
                data,
            ),
        ]
    );
}

#[test]
fn entry_points_error_before_create() {
    let env = Env::default();
    env.mock_all_auths();
    let c = VestingContractClient::new(&env, &env.register(VestingContract, ()));
    assert_eq!(c.try_claim(), Err(Ok(Error::NotInitialized)));
    assert_eq!(c.try_revoke(), Err(Ok(Error::NotInitialized)));
    assert_eq!(c.try_claimable(), Err(Ok(Error::NotInitialized)));
    assert_eq!(c.try_remaining(), Err(Ok(Error::NotInitialized)));
    assert_eq!(c.try_vested_amount(&START), Err(Ok(Error::NotInitialized)));
}

#[test]
#[should_panic]
fn create_requires_the_grantors_authorization() {
    let env = Env::default();
    let c = VestingContractClient::new(&env, &env.register(VestingContract, ()));
    c.create(
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &TOTAL,
        &START,
        &CLIFF,
        &DURATION,
        &true,
    );
}
