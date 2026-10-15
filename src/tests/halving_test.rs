use zenachain::blockchain::*;

#[test]
fn test_reward_era_0() {
    assert_eq!(Blockchain::calculate_reward(0), INITIAL_REWARD);
    assert_eq!(Blockchain::calculate_reward(1_000_000), INITIAL_REWARD);
}

#[test]
fn test_reward_era_1() {
    let r = Blockchain::calculate_reward(HALVING_INTERVAL_BLOCKS);
    assert_eq!(r, INITIAL_REWARD / 2);
}

#[test]
fn test_reward_era_2() {
    let r = Blockchain::calculate_reward(HALVING_INTERVAL_BLOCKS * 2);
    assert_eq!(r, INITIAL_REWARD / 4);
}

#[test]
fn test_reward_maks_halving() {
    let r = Blockchain::calculate_reward(HALVING_INTERVAL_BLOCKS * 100);
    assert_eq!(r, 0);
}
