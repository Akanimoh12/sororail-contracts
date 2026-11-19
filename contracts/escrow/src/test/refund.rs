// ---------------------------------------------------------------- refund

use super::*;
use soroban_sdk::{testutils::Events as _, Event as _};

#[test]
fn refund_by_depositor_is_blocked_before_the_deadline() {
    let f = Fixture::funded(true);
    assert_eq!(
        f.client.try_refund(&f.depositor),
        Err(Ok(Error::DeadlineNotReached))
    );
    // Funds stay put.
    assert_eq!(f.escrow_balance(), AMOUNT);
}

#[test]
fn refund_by_depositor_succeeds_after_the_deadline() {
    let f = Fixture::funded(true);
    let before = f.token.balance(&f.depositor);
    f.advance_past_deadline();
    f.client.refund(&f.depositor);

    assert_eq!(f.token.balance(&f.depositor), before + AMOUNT);
    assert_eq!(f.escrow_balance(), 0);
    assert_eq!(f.client.state(), State::Refunded);
}

#[test]
fn refund_emits_refunded_event_with_correct_topics_and_data() {
    let f = Fixture::funded(true);
    f.advance_past_deadline();
    f.client.refund(&f.depositor);

    let all = f.env.events().all().filter_by_contract(&f.client.address);
    let events = all.events();
    // Refunded only (from the last invocation)
    assert_eq!(events.len(), 1);
    let expected = Refunded {
        depositor: f.depositor.clone(),
        amount: AMOUNT,
        refunded_by: f.depositor.clone(),
    };
    assert_eq!(&events[0], &expected.to_xdr(&f.env, &f.client.address));
}

#[test]
fn refund_by_arbiter_is_allowed_before_the_deadline() {
    let f = Fixture::funded(true);
    f.client.refund(&f.arbiter);
    assert_eq!(f.escrow_balance(), 0);
    assert_eq!(f.client.state(), State::Refunded);
}

#[test]
fn refund_is_not_callable_by_the_beneficiary_or_an_outsider() {
    let f = Fixture::funded(true);
    f.advance_past_deadline();
    assert_eq!(
        f.client.try_refund(&f.beneficiary),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        f.client.try_refund(&f.outsider),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn refund_at_exactly_the_deadline_is_allowed() {
    // Boundary: the deadline is inclusive of the refund right.
    let f = Fixture::funded(true);
    f.env.ledger().with_mut(|l| l.timestamp = DEADLINE);
    f.client.refund(&f.depositor);
    assert_eq!(f.client.state(), State::Refunded);
}
