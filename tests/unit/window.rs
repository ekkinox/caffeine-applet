//! Unit tests for `src/window.rs`.
//!
//! Loaded by the `#[path]` declaration at the bottom of that file, so
//! `use super::*` gives access to private functions like `build_what`
//! and `acquire_inhibit`.
//!
//! Run with: cargo test

use super::*;

//  build_what ─

#[test]
fn build_what_without_lid_is_idle_sleep() {
    assert_eq!(build_what(false), "idle:sleep");
}

#[test]
fn build_what_with_lid_appends_handle_lid_switch() {
    assert_eq!(build_what(true), "idle:sleep:handle-lid-switch");
}

#[test]
fn build_what_without_lid_does_not_contain_lid_segment() {
    assert!(!build_what(false).contains("handle-lid-switch"));
}

//  CaffeineConfig ─

#[test]
fn caffeine_config_default_does_not_inhibit_lid() {
    assert!(!CaffeineConfig::default().inhibit_lid);
}

#[test]
fn caffeine_config_serde_round_trips_true() {
    let original = CaffeineConfig { inhibit_lid: true };
    let json = serde_json::to_string(&original).expect("serialise");
    let restored: CaffeineConfig = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(restored.inhibit_lid, original.inhibit_lid);
}

#[test]
fn caffeine_config_serde_round_trips_false() {
    let original = CaffeineConfig { inhibit_lid: false };
    let json = serde_json::to_string(&original).expect("serialise");
    let restored: CaffeineConfig = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(restored.inhibit_lid, original.inhibit_lid);
}

#[test]
fn caffeine_config_tolerates_unknown_fields() {
    // deny_unknown_fields is not set, so forward-compat should hold.
    let json = r#"{"inhibit_lid": false, "future_field": 42}"#;
    assert!(serde_json::from_str::<CaffeineConfig>(json).is_ok());
}

//  build_what + config interaction

#[test]
fn what_string_from_default_config_is_idle_sleep() {
    assert_eq!(
        build_what(CaffeineConfig::default().inhibit_lid),
        "idle:sleep"
    );
}

#[test]
fn what_string_from_lid_config_contains_all_three_segments() {
    let what = build_what(CaffeineConfig { inhibit_lid: true }.inhibit_lid);
    assert!(what.contains("idle"));
    assert!(what.contains("sleep"));
    assert!(what.contains("handle-lid-switch"));
}

//  acquire_inhibit (logind required)

/// Requires a running `org.freedesktop.login1` on the system bus.
/// Run with: cargo test -- --include-ignored
#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn acquire_inhibit_without_lid_returns_valid_fd() {
    let fd = acquire_inhibit(false).expect("should acquire inhibit fd");
    drop(fd);
}

#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn acquire_inhibit_with_lid_returns_valid_fd() {
    let fd = acquire_inhibit(true).expect("should acquire inhibit fd with lid");
    drop(fd);
}

#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn inhibit_fd_is_open_while_held_and_released_on_drop() {
    let fd = acquire_inhibit(false).expect("acquire");
    assert!(fd.try_clone().is_ok(), "fd should be open while held");
    drop(fd);
    // Kernel guarantees the inhibit lock is released when the fd is closed.
}

