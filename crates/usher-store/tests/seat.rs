//! Story 1.5, PR D: the seat lines of a run are stored beside its sources. A name is a
//! path of segments, so a server named `a.b` is one segment and never two. Membership
//! only: no field holds a value. Public API.

use usher_store::{RunRecord, SeatRecord, SeatRow, Store};
use usher_testkit::ScratchHome;

fn run() -> RunRecord<'static> {
    RunRecord {
        recorded_at: "2026-11-20T12:00:00Z",
        harness: "claude-code",
        harness_version: Some("2.1.317"),
        knowledge_schema: 4,
        knowledge_sequence: 5,
        environment_key: "env-key-1",
    }
}

fn seg(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| (*p).to_string()).collect()
}

#[test]
fn seat_lines_round_trip_with_each_name_kept_as_its_segments() {
    let scratch = ScratchHome::new().expect("scratch");
    let mut store = Store::open(&scratch.home().join(".local/state/usher")).expect("a store");
    let names = vec![
        seg(&["mcpServers", "a.b"]),
        seg(&["mcpServers", "a.b", "env", "K"]),
        seg(&["mcpServers", "z"]),
    ];
    let keys = vec![seg(&["oauthAccount"])];
    let seat = [
        SeatRecord {
            part: "mcp",
            item_id: "claude-code.seat-mcp-a",
            present: true,
            names: &names,
        },
        SeatRecord {
            part: "packs",
            item_id: "claude-code.seat-packs",
            present: false,
            names: &[],
        },
        SeatRecord {
            part: "auth-route",
            item_id: "claude-code.seat-auth-account",
            present: true,
            names: &keys,
        },
    ];
    let id = store
        .record_inventory_with_seat(&run(), &[], &seat)
        .expect("a run with seat lines");
    let rows = store.seat_of(id).expect("seat rows");
    assert_eq!(
        rows,
        vec![
            SeatRow {
                part: "mcp".to_string(),
                item_id: "claude-code.seat-mcp-a".to_string(),
                present: true,
                names: names.clone(),
            },
            SeatRow {
                part: "packs".to_string(),
                item_id: "claude-code.seat-packs".to_string(),
                present: false,
                names: Vec::new(),
            },
            SeatRow {
                part: "auth-route".to_string(),
                item_id: "claude-code.seat-auth-account".to_string(),
                present: true,
                names: keys,
            },
        ]
    );
    assert_eq!(
        rows[0].names[0],
        ["mcpServers", "a.b"],
        "one segment, not two"
    );
}

#[test]
fn a_second_run_keeps_its_own_seat_lines() {
    let scratch = ScratchHome::new().expect("scratch");
    let mut store = Store::open(&scratch.home().join(".local/state/usher")).expect("a store");
    let one = [SeatRecord {
        part: "mcp",
        item_id: "claude-code.seat-mcp-a",
        present: true,
        names: &[],
    }];
    let two = [SeatRecord {
        part: "mcp",
        item_id: "claude-code.seat-mcp-a",
        present: false,
        names: &[],
    }];
    let first = store
        .record_inventory_with_seat(&run(), &[], &one)
        .expect("one");
    let second = store
        .record_inventory_with_seat(&run(), &[], &two)
        .expect("two");
    assert_ne!(first, second);
    assert!(store.seat_of(first).expect("rows")[0].present);
    assert!(!store.seat_of(second).expect("rows")[0].present);
}

#[test]
fn a_run_with_no_seat_lines_has_none() {
    let scratch = ScratchHome::new().expect("scratch");
    let mut store = Store::open(&scratch.home().join(".local/state/usher")).expect("a store");
    let id = store.record_inventory(&run(), &[]).expect("a run");
    assert!(store.seat_of(id).expect("rows").is_empty());
}
