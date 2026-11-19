// ------------------------------------------------------------------ revoke

use super::*;
use soroban_sdk::{testutils::Events as _, Event as _};

#[test]
fn revoke_returns_only_the_unvested_portion() {
    let f = Fixture::new(true);
    let grantor_before = f.token.balance(&f.grantor);
    f.at(START + 300);
    assert_eq!(f.client.revoke(), TOTAL * 7 / 10);

    assert_eq!(f.token.balance(&f.grantor), grantor_before + TOTAL * 7 / 10);
    // The vested 30% stays in the contract for the beneficiary.
    assert_eq!(f.held(), TOTAL * 3 / 10);
    f.assert_conserved();
}

#[test]
fn the_vested_portion_stays_claimable_after_revocation() {
    let f = Fixture::new(true);
    f.at(START + 300);
    f.client.revoke();
    f.at(START + 900);
    assert_eq!(f.client.claim(), TOTAL * 3 / 10);
    assert_eq!(f.token.balance(&f.beneficiary), TOTAL * 3 / 10);
    assert_eq!(f.held(), 0);
    f.assert_conserved();
}

#[test]
fn revoke_accounts_for_what_was_already_claimed() {
    let f = Fixture::new(true);
    f.at(START + 200);
    f.client.claim();
    f.at(START + 600);
    f.client.revoke();

    // 60% vested overall, 20% already taken, 40% still claimable, 40% back.
    assert_eq!(f.client.claimable(), TOTAL * 4 / 10);
    f.assert_conserved();
    f.client.claim();
    assert_eq!(f.token.balance(&f.beneficiary), TOTAL * 6 / 10);
    assert_eq!(f.held(), 0);
    f.assert_conserved();
}

#[test]
fn revoke_before_the_cliff_returns_everything() {
    let f = Fixture::new(true);
    let grantor_before = f.token.balance(&f.grantor);
    assert_eq!(f.client.revoke(), TOTAL);
    assert_eq!(f.token.balance(&f.grantor), grantor_before + TOTAL);
    assert_eq!(f.held(), 0);
    // Nothing ever vested, so there is nothing to claim.
    assert_eq!(f.client.try_claim(), Err(Ok(Error::VestingCliffNotReached)));
    f.assert_conserved();
}

#[test]
fn revoke_before_the_cliff_then_advance_time_and_claim() {
    let f = Fixture::new(true);
    // Revoke before the cliff.
    f.client.revoke();
    // Advance time past the cliff -- but revoked_at freezes effective time.
    f.at(START + CLIFF + 1_000);
    // After revocation before the cliff, claim still reports CliffNotReached
    // because effective time is frozen at revoked_at (before the cliff).
    // Nothing was ever vested, so nothing is claimable.
    assert_eq!(f.client.try_claim(), Err(Ok(Error::VestingCliffNotReached)));
    f.assert_conserved();
}

#[test]
fn revoke_after_the_end_returns_nothing() {
    let f = Fixture::new(true);
    f.at(START + DURATION + 500);
    assert_eq!(f.client.revoke(), 0);
    assert_eq!(f.client.claimable(), TOTAL);
    f.assert_conserved();
}

#[test]
fn revoke_is_rejected_on_a_non_revocable_grant() {
    let f = Fixture::new(false);
    f.at(START + 300);
    assert_eq!(f.client.try_revoke(), Err(Ok(Error::VestingNotRevocable)));
    assert_eq!(f.held(), TOTAL);
}

#[test]
fn revoke_rejects_a_second_call() {
    let f = Fixture::new(true);
    f.at(START + 300);
    f.client.revoke();
    assert_eq!(f.client.try_revoke(), Err(Ok(Error::VestingRevoked)));
}

#[test]
#[should_panic]
fn revoke_requires_the_grantors_authorization() {
    let f = Fixture::new(true);
    f.at(START + 300);
    f.env.set_auths(&[]);
    f.client.revoke();
}

#[test]
fn revoke_emits_the_revoked_event() {
    let f = Fixture::new(true);
    f.at(START + 300);
    f.client.revoke();

    // At t = START + 300 with DURATION = 1_000 and TOTAL = 1_000_000:
    // vested = TOTAL * 300 / 1000 = 300_000 (still claimable)
    // returned = TOTAL * 700 / 1000 = 700_000
    let expected = Revoked {
        grantor: f.grantor.clone(),
        returned_to_grantor: TOTAL * 7 / 10,
        still_claimable: TOTAL * 3 / 10,
        revoked_at: START + 300,
    };
    // filter_by_contract returns ContractEvents; .events() gives &[ContractEvent].
    // The contract emits `created` at setup and `revoked` here — take the last one.
    let all = f.env.events().all().filter_by_contract(&f.client.address);
    let events = all.events();
    let last = events.last().expect("no events emitted");
    assert_eq!(last, &expected.to_xdr(&f.env, &f.client.address));
}
