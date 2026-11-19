// --------------------------------------------------------------- dispute

use super::*;
use soroban_sdk::{testutils::Events as _, Event as _};

#[test]
fn dispute_requires_an_arbiter() {
    let f = Fixture::funded(false);
    assert_eq!(
        f.client.try_dispute(&f.depositor),
        Err(Ok(Error::EscrowNoArbiter))
    );
}

#[test]
fn dispute_is_callable_by_either_party() {
    for caller_is_depositor in [true, false] {
        let f = Fixture::funded(true);
        let caller = if caller_is_depositor {
            f.depositor.clone()
        } else {
            f.beneficiary.clone()
        };
        f.client.dispute(&caller);
        assert_eq!(f.client.state(), State::Disputed);
    }
}

#[test]
fn dispute_is_not_callable_by_the_arbiter_or_an_outsider() {
    let f = Fixture::funded(true);
    assert_eq!(
        f.client.try_dispute(&f.arbiter),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        f.client.try_dispute(&f.outsider),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn dispute_rejects_a_second_call() {
    let f = Fixture::funded(true);
    f.client.dispute(&f.depositor);
    assert_eq!(
        f.client.try_dispute(&f.depositor),
        Err(Ok(Error::EscrowAlreadyDisputed))
    );
}

#[test]
fn dispute_requires_funding_first() {
    let f = Fixture::new(true);
    assert_eq!(
        f.client.try_dispute(&f.depositor),
        Err(Ok(Error::EscrowNotFunded))
    );
}

#[test]
fn dispute_emits_disputed_event_with_correct_topics_and_data() {
    let f = Fixture::funded(true);
    f.client.dispute(&f.depositor);

    let all = f.env.events().all().filter_by_contract(&f.client.address);
    let events = all.events();
    // Disputed only (from the last invocation)
    assert_eq!(events.len(), 1);

    let expected = Disputed {
        raised_by: f.depositor.clone(),
    };
    assert_eq!(&events[0], &expected.to_xdr(&f.env, &f.client.address));
}

#[test]
fn release_and_refund_are_blocked_while_disputed() {
    let f = Fixture::funded(true);
    f.client.dispute(&f.depositor);
    assert_eq!(
        f.client.try_release(&f.depositor),
        Err(Ok(Error::EscrowNotFunded))
    );
    f.advance_past_deadline();
    assert_eq!(
        f.client.try_refund(&f.depositor),
        Err(Ok(Error::EscrowNotFunded))
    );
    assert_eq!(f.escrow_balance(), AMOUNT);
}
