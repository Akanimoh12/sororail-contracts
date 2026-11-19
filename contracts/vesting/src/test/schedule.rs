// ----------------------------------------- the schedule (pure, no ledger)

use super::*;

#[test]
fn nothing_vests_before_the_cliff() {
    let env = Env::default();
    let g = bare(&env, CLIFF, DURATION);
    assert_eq!(g.vested_amount(0), Ok(0));
    assert_eq!(g.vested_amount(START), Ok(0));
    assert_eq!(g.vested_amount(START + CLIFF - 1), Ok(0));
}

#[test]
fn the_cliff_vests_the_elapsed_proportion_in_one_step() {
    let env = Env::default();
    let g = bare(&env, CLIFF, DURATION);
    // At the cliff, 100/1000 of the schedule has elapsed.
    assert_eq!(g.vested_amount(START + CLIFF), Ok(TOTAL / 10));
}

#[test]
fn vesting_is_linear_after_the_cliff() {
    let env = Env::default();
    let g = bare(&env, CLIFF, DURATION);
    assert_eq!(g.vested_amount(START + 250), Ok(TOTAL / 4));
    assert_eq!(g.vested_amount(START + 500), Ok(TOTAL / 2));
    assert_eq!(g.vested_amount(START + 750), Ok(TOTAL * 3 / 4));
}

#[test]
fn fully_vested_at_start_plus_duration() {
    let env = Env::default();
    let g = bare(&env, CLIFF, DURATION);
    assert_eq!(g.vested_amount(START + DURATION), Ok(TOTAL));
    assert_eq!(g.vested_amount(START + DURATION + 1), Ok(TOTAL));
    assert_eq!(g.vested_amount(u64::MAX), Ok(TOTAL));
}

#[test]
fn a_cliff_equal_to_duration_is_all_or_nothing() {
    let env = Env::default();
    let g = bare(&env, DURATION, DURATION);
    assert_eq!(g.vested_amount(START + DURATION - 1), Ok(0));
    assert_eq!(g.vested_amount(START + DURATION), Ok(TOTAL));
}

#[test]
fn a_zero_cliff_vests_linearly_from_the_start() {
    let env = Env::default();
    let g = bare(&env, 0, DURATION);
    assert_eq!(g.vested_amount(START), Ok(0));
    assert_eq!(g.vested_amount(START + 1), Ok(TOTAL / DURATION as i128));
    assert_eq!(g.vested_amount(START + 500), Ok(TOTAL / 2));
}

#[test]
fn a_one_second_schedule_vests_in_one_tick() {
    let env = Env::default();
    let g = bare(&env, 0, 1);
    assert_eq!(g.vested_amount(START), Ok(0));
    assert_eq!(g.vested_amount(START + 1), Ok(TOTAL));
}

#[test]
fn revocation_freezes_the_schedule() {
    let env = Env::default();
    let mut g = bare(&env, CLIFF, DURATION);
    g.revoked_at = Some(START + 300);
    assert_eq!(g.vested_amount(START + 300), Ok(TOTAL * 3 / 10));
    assert_eq!(g.vested_amount(START + 900), Ok(TOTAL * 3 / 10));
    assert_eq!(g.vested_amount(u64::MAX), Ok(TOTAL * 3 / 10));
}

#[test]
fn vesting_reports_overflow_rather_than_wrapping() {
    let env = Env::default();
    let mut g = bare(&env, 0, DURATION);
    g.total = i128::MAX;
    assert_eq!(g.vested_amount(START + 500), Err(Error::Overflow));
}

#[test]
fn long_duration_large_total_stays_inside_documented_safe_range() {
    let env = Env::default();
    let ten_years = 10 * 365 * 24 * 60 * 60;
    let mut g = bare(&env, 0, ten_years);

    // One billion 18-decimal tokens over ten years stays well below i128::MAX
    // during the checked `total * elapsed / duration` calculation.
    g.total = 1_000_000_000_i128 * 1_000_000_000_000_000_000_i128;

    assert_eq!(g.vested_amount(START + ten_years / 2), Ok(g.total / 2));
    assert!(g.vested_amount(START + ten_years - 1).is_ok());
    assert_eq!(g.vested_amount(START + ten_years), Ok(g.total));
}
