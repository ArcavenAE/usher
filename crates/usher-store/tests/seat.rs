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
    let keys = vec![("oauthAccount".to_string(), true)];
    let seat = [
        SeatRecord {
            part: "mcp",
            item_id: "claude-code.seat-mcp-a",
            present: Some(true),
            names: &names,
            members: &[],
            note: None,
        },
        SeatRecord {
            part: "packs",
            item_id: "claude-code.seat-packs",
            present: Some(false),
            names: &[],
            members: &[],
            note: None,
        },
        SeatRecord {
            part: "auth-route",
            item_id: "claude-code.seat-auth-account",
            present: Some(true),
            names: &[],
            members: &keys,
            note: None,
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
                present: Some(true),
                names: names.clone(),
                members: Vec::new(),
                note: None,
            },
            SeatRow {
                part: "packs".to_string(),
                item_id: "claude-code.seat-packs".to_string(),
                present: Some(false),
                names: Vec::new(),
                members: Vec::new(),
                note: None,
            },
            SeatRow {
                part: "auth-route".to_string(),
                item_id: "claude-code.seat-auth-account".to_string(),
                present: Some(true),
                names: Vec::new(),
                members: keys,
                note: None,
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
        present: Some(true),
        names: &[],
        members: &[],
        note: None,
    }];
    let two = [SeatRecord {
        part: "mcp",
        item_id: "claude-code.seat-mcp-a",
        present: Some(false),
        names: &[],
        members: &[],
        note: None,
    }];
    let first = store
        .record_inventory_with_seat(&run(), &[], &one)
        .expect("one");
    let second = store
        .record_inventory_with_seat(&run(), &[], &two)
        .expect("two");
    assert_ne!(first, second);
    assert_eq!(store.seat_of(first).expect("rows")[0].present, Some(true));
    assert_eq!(store.seat_of(second).expect("rows")[0].present, Some(false));
}

#[test]
fn a_run_with_no_seat_lines_has_none() {
    let scratch = ScratchHome::new().expect("scratch");
    let mut store = Store::open(&scratch.home().join(".local/state/usher")).expect("a store");
    // A run with seat lines comes first, so a reader that returned every row would show
    // them under the second run too.
    let with = [SeatRecord {
        part: "mcp",
        item_id: "claude-code.seat-mcp-a",
        present: Some(true),
        names: &[],
        members: &[],
        note: None,
    }];
    let first = store
        .record_inventory_with_seat(&run(), &[], &with)
        .expect("a run with a seat line");
    let second = store.record_inventory(&run(), &[]).expect("a run");
    assert_eq!(store.seat_of(first).expect("rows").len(), 1);
    assert!(store.seat_of(second).expect("rows").is_empty());
}

#[test]
fn environment_and_key_members_round_trip_as_pairs_with_absent_ones_included() {
    let scratch = ScratchHome::new().expect("scratch");
    let mut store = Store::open(&scratch.home().join(".local/state/usher")).expect("a store");
    let vars = vec![
        ("FLEET_API_KEY".to_string(), false),
        ("FLEET_TOKEN".to_string(), true),
    ];
    let keys = vec![("oauthAccount".to_string(), false)];
    let seat = [
        SeatRecord {
            part: "auth-route",
            item_id: "claude-code.seat-auth-env",
            present: None,
            names: &[],
            members: &vars,
            note: None,
        },
        SeatRecord {
            part: "auth-route",
            item_id: "claude-code.seat-auth-account",
            present: Some(false),
            names: &[],
            members: &keys,
            note: None,
        },
    ];
    let id = store
        .record_inventory_with_seat(&run(), &[], &seat)
        .expect("a run");
    let rows = store.seat_of(id).expect("rows");
    assert_eq!(rows[0].present, None);
    assert_eq!(rows[0].members, vars);
    assert_eq!(rows[1].present, Some(false));
    assert_eq!(rows[1].members, keys);
}

/// The derived-state allowlist grows by the seat table, with these columns, and no cell of
/// it holds anything but the part, the item id, the flags, and the names the caller
/// passed. The older test that pins two tables becomes a meaning change in the code step.
#[test]
fn the_seat_table_joins_the_derived_state_allowlist_and_holds_names_only() {
    let scratch = ScratchHome::new().expect("scratch");
    let dir = scratch.home().join(".local/state/usher");
    let mut store = Store::open(&dir).expect("a store");
    let names = vec![seg(&["mcpServers", "a.b"])];
    let members = vec![("FLEET_TOKEN".to_string(), true)];
    let seat = [
        SeatRecord {
            part: "mcp",
            item_id: "claude-code.seat-mcp-launch-dir",
            present: Some(true),
            names: &names,
            members: &[],
            note: None,
        },
        SeatRecord {
            part: "auth-route",
            item_id: "claude-code.seat-auth-env",
            present: None,
            names: &[],
            members: &members,
            note: None,
        },
        SeatRecord {
            part: "auth-route",
            item_id: "claude-code.seat-auth-account",
            present: None,
            names: &[],
            members: &[],
            note: Some("CLAUDE_CONFIG_DIR is not absolute"),
        },
    ];
    store
        .record_inventory_with_seat(&run(), &[], &seat)
        .expect("a run");
    let conn = rusqlite::Connection::open(dir.join("usher.db")).expect("a second connection");
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .expect("prepare")
        .query_map([], |r| r.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    assert_eq!(
        tables,
        [
            "inventory_run",
            "inventory_seat",
            "inventory_source",
            "map_setting"
        ]
    );
    let columns: Vec<String> = conn
        .prepare("SELECT name FROM pragma_table_info('inventory_seat') ORDER BY cid")
        .expect("prepare")
        .query_map([], |r| r.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    assert_eq!(
        columns,
        [
            "run_id", "part", "item_id", "present", "names", "members", "note"
        ]
    );
    let cells: Vec<String> = conn
        .prepare("SELECT part || '|' || item_id || '|' || ifnull(present, 'null') || '|' || names || '|' || members || '|' || ifnull(note, 'null') FROM inventory_seat ORDER BY rowid")
        .expect("prepare")
        .query_map([], |r| r.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    assert_eq!(
        cells,
        [
            r#"mcp|claude-code.seat-mcp-launch-dir|1|[["mcpServers","a.b"]]|[]|null"#,
            r#"auth-route|claude-code.seat-auth-env|null|[]|[["FLEET_TOKEN",true]]|null"#,
            "auth-route|claude-code.seat-auth-account|null|[]|[]|CLAUDE_CONFIG_DIR is not absolute",
        ]
    );
}

#[test]
fn a_note_round_trips_with_no_flag_and_empty_names_and_members() {
    let scratch = ScratchHome::new().expect("scratch");
    let mut store = Store::open(&scratch.home().join(".local/state/usher")).expect("a store");
    let seat = [SeatRecord {
        part: "auth-route",
        item_id: "claude-code.seat-auth-account",
        present: None,
        names: &[],
        members: &[],
        note: Some("CLAUDE_CONFIG_DIR is not absolute"),
    }];
    let id = store
        .record_inventory_with_seat(&run(), &[], &seat)
        .expect("a run");
    let rows = store.seat_of(id).expect("rows");
    assert_eq!(rows[0].present, None);
    assert_eq!(
        rows[0].note.as_deref(),
        Some("CLAUDE_CONFIG_DIR is not absolute")
    );
    assert!(rows[0].names.is_empty() && rows[0].members.is_empty());
}
