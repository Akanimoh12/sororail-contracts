// ------------------------------------------------------------------ fund

use super::*;
use soroban_sdk::{testutils::Events as _, Event as _};

#[test]
fn fund_moves_the_tokens_into_the_contract() {
    let f = Fixture::new(true);
    let before = f.token.balance(&f.depositor);
    f.client.fund();

    assert_eq!(f.escrow_balance(), AMOUNT);
    assert_eq!(f.token.balance(&f.depositor), before - AMOUNT);
    assert_eq!(f.client.state(), State::Funded);
}

#[test]
fn fund_rejects_a_second_call() {
    let f = Fixture::funded(true);
    assert_eq!(f.client.try_fund(), Err(Ok(Error::EscrowNotFundable)));
}

/// `fund` writes `State::Funded` before the token pull. If the depositor cannot
/// pay, the host reverts the invocation so a later `state()` is still `Created`.
#[test]
fn fund_reverts_cleanly_when_depositor_cannot_pay() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = START_TS);

    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_address = sac.address();
    // Deliberately mint nothing to the depositor.
    let token = TokenClient::new(&env, &token_address);

    let client = EscrowContractClient::new(&env, &env.register(EscrowContract, ()));
    client.init(
        &depositor,
        &beneficiary,
        &Some(arbiter),
        &token_address,
        &AMOUNT,
        &DEADLINE,
    );
    assert_eq!(client.state(), State::Created);

    assert!(client.try_fund().is_err());
    assert_eq!(
        client.state(),
        State::Created,
        "optimistic Funded write leaked"
    );
    assert_eq!(token.balance(&client.address), 0);
    assert_eq!(client.get().amount, AMOUNT);
}

#[test]
fn fund_rejects_a_closed_escrow() {
    let f = Fixture::funded(true);
    f.client.release(&f.depositor);
    assert_eq!(f.client.try_fund(), Err(Ok(Error::EscrowClosed)));
}

#[test]
fn fund_emits_funded_event_with_correct_topics_and_data() {
    let f = Fixture::new(true);
    f.client.fund();

    let all = f.env.events().all().filter_by_contract(&f.client.address);
    let events = all.events();
    assert_eq!(events.len(), 1); // Funded only (from the last invocation)

    let expected = Funded {
        depositor: f.depositor.clone(),
        amount: AMOUNT,
    };
    assert_eq!(&events[0], &expected.to_xdr(&f.env, &f.client.address));
}
