// --------------------------------------------------------------- release

use super::*;
use soroban_sdk::{testutils::Events as _, Event as _};

#[test]
fn release_pays_the_beneficiary() {
    let f = Fixture::funded(true);
    f.client.release(&f.depositor);

    assert_eq!(f.token.balance(&f.beneficiary), AMOUNT);
    assert_eq!(f.escrow_balance(), 0);
    assert_eq!(f.client.state(), State::Released);
}

#[test]
fn release_emits_released_event_with_correct_topics_and_data() {
    let f = Fixture::funded(true);
    f.client.release(&f.depositor);

    let all = f.env.events().all().filter_by_contract(&f.client.address);
    let events = all.events();
    // Released only (from the last invocation)
    assert_eq!(events.len(), 1);
    let expected = Released {
        beneficiary: f.beneficiary.clone(),
        amount: AMOUNT,
        released_by: f.depositor.clone(),
    };
    assert_eq!(&events[0], &expected.to_xdr(&f.env, &f.client.address));
}

#[test]
fn release_is_callable_by_the_arbiter() {
    let f = Fixture::funded(true);
    f.client.release(&f.arbiter);
    assert_eq!(f.token.balance(&f.beneficiary), AMOUNT);
}

#[test]
fn release_is_not_callable_by_the_beneficiary() {
    // The point of the escrow: the payee cannot pay themselves.
    let f = Fixture::funded(true);
    assert_eq!(
        f.client.try_release(&f.beneficiary),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn release_is_not_callable_by_an_outsider() {
    let f = Fixture::funded(true);
    assert_eq!(
        f.client.try_release(&f.outsider),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn release_requires_funding_first() {
    let f = Fixture::new(true);
    assert_eq!(
        f.client.try_release(&f.depositor),
        Err(Ok(Error::EscrowNotFunded))
    );
}

#[test]
fn release_rejects_a_closed_escrow() {
    let f = Fixture::funded(true);
    f.client.release(&f.depositor);
    assert_eq!(
        f.client.try_release(&f.depositor),
        Err(Ok(Error::EscrowClosed))
    );
}
