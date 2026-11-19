// --------------------------------------------------------------- resolve

use super::*;
use soroban_sdk::{testutils::Events as _, Event as _};

#[test]
fn resolve_splits_between_the_parties() {
    let f = Fixture::funded(true);
    f.client.dispute(&f.beneficiary);
    f.client.resolve(&2_500);

    assert_eq!(f.token.balance(&f.beneficiary), AMOUNT / 4);
    assert_eq!(f.escrow_balance(), 0);
    assert_eq!(f.client.state(), State::Resolved);
}

#[test]
fn resolve_requires_a_dispute() {
    let f = Fixture::funded(true);
    assert_eq!(
        f.client.try_resolve(&5_000),
        Err(Ok(Error::EscrowNotDisputed))
    );
}

#[test]
fn resolve_rejects_basis_points_above_the_maximum() {
    let f = Fixture::funded(true);
    f.client.dispute(&f.depositor);
    assert_eq!(
        f.client.try_resolve(&10_001),
        Err(Ok(Error::InvalidBasisPoints))
    );
    // Still resolvable afterwards -- the failed call changed nothing.
    assert_eq!(f.client.state(), State::Disputed);
}

#[test]
fn resolve_endpoints_pay_one_side_in_full() {
    let f = Fixture::funded(true);
    f.client.dispute(&f.depositor);
    f.client.resolve(&10_000);
    assert_eq!(f.token.balance(&f.beneficiary), AMOUNT);

    let g = Fixture::funded(true);
    let depositor_before = g.token.balance(&g.depositor);
    g.client.dispute(&g.depositor);
    g.client.resolve(&0);
    assert_eq!(g.token.balance(&g.depositor), depositor_before + AMOUNT);
    assert_eq!(g.token.balance(&g.beneficiary), 0);
}

#[test]
fn resolve_conserves_the_escrowed_amount_at_every_split() {
    // The invariant: paid_to_beneficiary + paid_to_depositor == amount,
    // exactly, including at splits that divide badly.
    for bps in [0u32, 1, 3, 333, 1_234, 5_000, 6_667, 9_999, 10_000] {
        let f = Fixture::funded(true);
        let depositor_before = f.token.balance(&f.depositor);
        f.client.dispute(&f.depositor);
        f.client.resolve(&bps);

        let to_beneficiary = f.token.balance(&f.beneficiary);
        let to_depositor = f.token.balance(&f.depositor) - depositor_before;

        assert_eq!(
            to_beneficiary + to_depositor,
            AMOUNT,
            "{bps}bps split leaked value"
        );
        assert_eq!(f.escrow_balance(), 0, "{bps}bps split stranded funds");
        assert!(to_beneficiary >= 0 && to_depositor >= 0);
    }
}

#[test]
fn resolve_rejects_a_second_call() {
    let f = Fixture::funded(true);
    f.client.dispute(&f.depositor);
    f.client.resolve(&5_000);
    assert_eq!(
        f.client.try_resolve(&5_000),
        Err(Ok(Error::EscrowNotDisputed))
    );
}

#[test]
fn resolve_emits_resolved_event_with_correct_topics_and_data() {
    let f = Fixture::funded(true);
    f.client.dispute(&f.depositor);
    f.client.resolve(&2_500);

    let all = f.env.events().all().filter_by_contract(&f.client.address);
    let events = all.events();
    // Resolved only (from the last invocation)
    assert_eq!(events.len(), 1);

    let expected = Resolved {
        arbiter: f.arbiter.clone(),
        split_bps: 2_500,
        to_beneficiary: AMOUNT / 4,
        to_depositor: AMOUNT * 3 / 4,
    };
    assert_eq!(&events[0], &expected.to_xdr(&f.env, &f.client.address));
}
