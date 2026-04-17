//! Integration tests for the D-Bus / logind inhibit layer.
//!
//! These tests call `org.freedesktop.login1` on the **system** D-Bus and
//! require either `systemd-logind` or `elogind` to be running.
//! They are marked `#[ignore]` so a plain `cargo test` skips them.
//!
//! # Running locally
//!
//! ```sh
//! cargo test --test dbus -- --include-ignored
//! ```
//!
//! # Running in CI (self-hosted / systemd container)
//!
//! Set the repository variable `LOGIND_AVAILABLE=1` and the workflow
//! will pass `--include-ignored` automatically.

use std::os::fd::OwnedFd;
use zbus::blocking::Connection;
use zbus::zvariant::OwnedFd as ZbusFd;

//  helpers ─

fn system_bus() -> Connection {
    Connection::system().expect("could not connect to the system D-Bus — is dbus-daemon running?")
}

fn call_inhibit(conn: &Connection, what: &str, who: &str, why: &str) -> OwnedFd {
    let zfd: ZbusFd = conn
        .call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "Inhibit",
            &(what, who, why, "block"),
        )
        .unwrap_or_else(|e| panic!("Inhibit({what:?}) failed: {e}"))
        .body()
        .deserialize()
        .expect("failed to deserialise inhibit fd");
    zfd.into()
}

//  tests ─

#[test]
#[ignore = "requires the system D-Bus (dbus-daemon)"]
fn system_dbus_is_reachable() {
    let _conn = system_bus();
}

#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn logind_responds_to_dbus_peer_ping() {
    let conn = system_bus();
    conn.call_method(
        Some("org.freedesktop.login1"),
        "/org/freedesktop/login1",
        Some("org.freedesktop.DBus.Peer"),
        "Ping",
        &(),
    )
    .expect("org.freedesktop.login1 did not respond to Peer.Ping");
}

#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn inhibit_idle_sleep_acquires_and_releases() {
    let conn = system_bus();
    let fd = call_inhibit(
        &conn,
        "idle:sleep",
        "caffeine-integration-test",
        "integration test",
    );
    drop(fd);
}

#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn inhibit_idle_sleep_lid_switch_acquires_and_releases() {
    let conn = system_bus();
    let fd = call_inhibit(
        &conn,
        "idle:sleep:handle-lid-switch",
        "caffeine-integration-test",
        "lid switch integration test",
    );
    drop(fd);
}

#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn two_concurrent_inhibit_fds_are_independent() {
    let conn = system_bus();
    let fd1 = call_inhibit(&conn, "idle", "caffeine-test-a", "concurrent test a");
    let fd2 = call_inhibit(&conn, "sleep", "caffeine-test-b", "concurrent test b");
    drop(fd1);
    assert!(
        fd2.try_clone().is_ok(),
        "fd2 should still be open after fd1 is dropped"
    );
    drop(fd2);
}

#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn repeated_acquire_release_cycle_does_not_error() {
    let conn = system_bus();
    for i in 0..5 {
        let why = format!("cycle {i}");
        let fd = call_inhibit(&conn, "idle:sleep", "caffeine-cycle-test", &why);
        drop(fd);
    }
}

