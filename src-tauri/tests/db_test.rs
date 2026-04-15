use rusqlite::{Connection, OptionalExtension, params};

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("in-memory DB");
    conn.execute_batch(
        "CREATE TABLE channels (
            id TEXT PRIMARY KEY, name TEXT NOT NULL, icon_url TEXT,
            visible INTEGER NOT NULL DEFAULT 1, sort_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE programs (
            id TEXT PRIMARY KEY, channel_id TEXT NOT NULL, title TEXT NOT NULL,
            description TEXT, category TEXT, start_time INTEGER NOT NULL,
            end_time INTEGER NOT NULL, fetched_at INTEGER NOT NULL
        );
        CREATE TABLE favourites (title TEXT PRIMARY KEY, added_at INTEGER NOT NULL);
        CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE INDEX idx_programs_time ON programs(start_time, end_time);
        CREATE INDEX idx_programs_channel ON programs(channel_id);",
    )
    .expect("schema creation");
    conn
}

fn insert_channel(conn: &Connection, id: &str, name: &str, visible: i32, sort_order: i32) {
    conn.execute(
        "INSERT INTO channels (id, name, icon_url, visible, sort_order) VALUES (?1, ?2, NULL, ?3, ?4)",
        params![id, name, visible, sort_order],
    )
    .unwrap();
}

fn insert_program(
    conn: &Connection,
    id: &str,
    channel_id: &str,
    title: &str,
    start: i64,
    end: i64,
) {
    conn.execute(
        "INSERT INTO programs (id, channel_id, title, description, category, start_time, end_time, fetched_at)
         VALUES (?1, ?2, ?3, NULL, NULL, ?4, ?5, 0)",
        params![id, channel_id, title, start, end],
    )
    .unwrap();
}

// --- Tests ---

#[test]
fn test_init_db_creates_tables() {
    let conn = setup_test_db();
    let table_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('channels','programs','favourites','settings')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(table_count, 4, "all 4 tables must exist");
}

#[test]
fn test_upsert_and_get_channels() {
    let conn = setup_test_db();
    insert_channel(&conn, "ch1", "BBC One", 1, 0);
    insert_channel(&conn, "ch2", "ITV", 1, 1);

    let mut stmt = conn
        .prepare("SELECT id, name FROM channels ORDER BY sort_order")
        .unwrap();
    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], ("ch1".into(), "BBC One".into()));
    assert_eq!(rows[1], ("ch2".into(), "ITV".into()));
}

#[test]
fn test_channel_visibility_toggle() {
    let conn = setup_test_db();
    insert_channel(&conn, "ch1", "BBC One", 1, 0);
    insert_channel(&conn, "ch2", "Hidden", 0, 1);
    insert_program(&conn, "p1", "ch1", "News", 100, 200);
    insert_program(&conn, "p2", "ch2", "Movie", 100, 200);

    // Only ch1 is visible — range query should return only p1
    let mut stmt = conn
        .prepare(
            "SELECT p.id FROM programs p
             JOIN channels c ON p.channel_id = c.id
             WHERE c.visible = 1 AND p.end_time > 50 AND p.start_time < 300
             ORDER BY p.start_time",
        )
        .unwrap();
    let ids: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(ids, vec!["p1"]);

    // Toggle ch2 visible, now both should appear
    conn.execute("UPDATE channels SET visible = 1 WHERE id = 'ch2'", [])
        .unwrap();
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM programs p JOIN channels c ON p.channel_id = c.id WHERE c.visible = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_programs_time_range_query() {
    let conn = setup_test_db();
    insert_channel(&conn, "ch1", "Test", 1, 0);

    // Timestamps (seconds):
    //   before:   start=0,   end=50   — ends before range start (100)
    //   overlap_start: start=50, end=150 — straddles range start
    //   inside:   start=120, end=180  — entirely inside range [100, 200]
    //   overlap_end: start=180, end=250 — straddles range end
    //   after:    start=210, end=300  — starts after range end (200)
    insert_program(&conn, "before", "ch1", "Before", 0, 50);
    insert_program(&conn, "overlap_start", "ch1", "OverlapStart", 50, 150);
    insert_program(&conn, "inside", "ch1", "Inside", 120, 180);
    insert_program(&conn, "overlap_end", "ch1", "OverlapEnd", 180, 250);
    insert_program(&conn, "after", "ch1", "After", 210, 300);

    let from = 100_i64;
    let to = 200_i64;

    let mut stmt = conn
        .prepare(
            "SELECT p.id FROM programs p
             JOIN channels c ON p.channel_id = c.id
             WHERE c.visible = 1 AND p.end_time > ?1 AND p.start_time < ?2
             ORDER BY p.start_time",
        )
        .unwrap();
    let ids: Vec<String> = stmt
        .query_map(params![from, to], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    assert_eq!(
        ids,
        vec!["overlap_start", "inside", "overlap_end"],
        "only programs overlapping [100, 200) should be returned"
    );
}

#[test]
fn test_toggle_favourite() {
    let conn = setup_test_db();
    let title = "Breaking News";

    // Not yet a favourite
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM favourites WHERE title = ?1",
            params![title],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);

    // Add
    conn.execute(
        "INSERT INTO favourites (title, added_at) VALUES (?1, 1000)",
        params![title],
    )
    .unwrap();
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM favourites WHERE title = ?1",
            params![title],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    // Remove
    conn.execute("DELETE FROM favourites WHERE title = ?1", params![title])
        .unwrap();
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM favourites WHERE title = ?1",
            params![title],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_settings_crud() {
    let conn = setup_test_db();
    let key = "epg_url";

    // Get non-existent key — should be None
    let val: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .unwrap();
    assert!(val.is_none());

    // Set
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, "https://example.com/epg.xml"],
    )
    .unwrap();
    let val: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(val, "https://example.com/epg.xml");

    // Update
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, "https://updated.com/epg.xml"],
    )
    .unwrap();
    let val: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(val, "https://updated.com/epg.xml");
}

#[test]
fn test_upsert_channels_preserves_visibility() {
    let conn = setup_test_db();

    // Insert initial channel, then hide it
    conn.execute(
        "INSERT INTO channels (id, name, icon_url, visible, sort_order) VALUES ('ch1', 'BBC One', NULL, 1, 0)",
        [],
    )
    .unwrap();
    conn.execute("UPDATE channels SET visible = 0 WHERE id = 'ch1'", [])
        .unwrap();

    let visible_before: i32 = conn
        .query_row("SELECT visible FROM channels WHERE id = 'ch1'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(
        visible_before, 0,
        "channel should be hidden before re-upsert"
    );

    // Re-upsert — same SQL as db.rs::upsert_channels: only update name and icon_url, NOT visible
    conn.execute(
        "INSERT INTO channels (id, name, icon_url, visible, sort_order)
         VALUES ('ch1', 'BBC One HD', NULL, 1, 0)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, icon_url = excluded.icon_url",
        [],
    )
    .unwrap();

    let (name, visible): (String, i32) = conn
        .query_row(
            "SELECT name, visible FROM channels WHERE id = 'ch1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(name, "BBC One HD", "name should be updated by upsert");
    assert_eq!(visible, 0, "visibility must NOT be reset by upsert");
}
