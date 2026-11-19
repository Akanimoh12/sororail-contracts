// ------------------------------------------------------- conservation

use super::*;

#[test]
fn conservation_holds_across_claim_then_revoke_then_claim() {
    let f = Fixture::new(true);
    f.assert_conserved();

    f.at(START + CLIFF);
    f.client.claim();
    f.assert_conserved();

    f.at(START + 450);
    f.client.claim();
    f.assert_conserved();

    f.at(START + 700);
    f.client.revoke();
    f.assert_conserved();

    f.client.claim();
    f.assert_conserved();

    let g = f.client.get();
    assert_eq!(f.held(), 0);
    assert_eq!(g.claimed + g.returned, g.total);
    assert_eq!(
        f.token.balance(&f.beneficiary) + f.token.balance(&f.grantor),
        MINT
    );
}

#[test]
fn a_schedule_that_divides_badly_still_conserves() {
    // 7 into 3 -- deliberately lossy proportions.
    let f = Fixture::with_schedule(true, 0, 3);
    f.at(START + 1);
    f.client.claim();
    f.assert_conserved();
    f.at(START + 2);
    f.client.claim();
    f.assert_conserved();
    f.at(START + 3);
    f.client.claim();
    f.assert_conserved();
    assert_eq!(f.token.balance(&f.beneficiary), TOTAL);
    assert_eq!(f.held(), 0);
}
