// Unit tests for the generated block registry. Run with: cargo test
use super::blocks_generated::*;

#[test]
fn total_states_matches_data() {
    assert_eq!(TOTAL_STATES, 35723);
    assert_eq!(MAX_STATE_ID, TOTAL_STATES - 1);
}

#[test]
fn air_is_state_zero() {
    let air = block_by_name("minecraft:air").expect("air missing");
    assert_eq!(air.base_id, 0);
    assert_eq!(air.default_id, 0);
}

#[test]
fn default_states_resolve() {
    assert_eq!(block_by_name("minecraft:stone").unwrap().default_id, 1);
    // grass_block has 2 states; default is 9, not the first state (8).
    let grass = block_by_name("minecraft:grass_block").unwrap();
    assert_eq!(grass.state_count, 2);
    assert_eq!(grass.default_id, 9);
}

#[test]
fn state_lookup_returns_owning_block() {
    let b = block_of_state(9).expect("state 9 should exist");
    assert_eq!(b.name, "minecraft:grass_block");
    assert_eq!(block_of_state(8).unwrap().name, "minecraft:grass_block");
}

#[test]
fn out_of_range_state_is_none() {
    assert!(block_of_state(TOTAL_STATES).is_none());
    assert!(block_of_state(u16::MAX).is_none());
}

#[test]
fn unknown_name_is_none() {
    assert!(block_by_name("minecraft:does_not_exist").is_none());
}

#[test]
fn ranges_are_contiguous_and_sorted() {
    let mut expected_base = 0u16;
    for b in BLOCKS {
        assert_eq!(b.base_id, expected_base, "gap or overlap before {}", b.name);
        assert!(b.state_count >= 1, "{} has no states", b.name);
        assert!(
            b.default_id >= b.base_id && b.default_id < b.base_id + b.state_count,
            "default of {} is outside its range",
            b.name
        );
        expected_base = b.base_id + b.state_count;
    }
    assert_eq!(expected_base, TOTAL_STATES);
}

#[test]
fn every_state_maps_back_to_its_block() {
    for b in BLOCKS {
        for s in b.base_id..b.base_id + b.state_count {
            assert_eq!(block_of_state(s).unwrap().name, b.name);
        }
    }
}
