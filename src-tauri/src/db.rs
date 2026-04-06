use crate::error::AppError;
use crate::models::{Channel, Favourite, Program};
use rusqlite::Connection;
use std::path::Path;

pub fn init_db(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS channels (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            icon_url    TEXT,
            visible     INTEGER NOT NULL DEFAULT 1,
            sort_order  INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS programs (
            id          TEXT PRIMARY KEY,
            channel_id  TEXT NOT NULL REFERENCES channels(id),
            title       TEXT NOT NULL,
            description TEXT,
            category    TEXT,
            start_time  INTEGER NOT NULL,
            end_time    INTEGER NOT NULL,
            fetched_at  INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS favourites (
            title       TEXT PRIMARY KEY,
            added_at    INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS settings (
            key         TEXT PRIMARY KEY,
            value       TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_programs_time ON programs(start_time, end_time);
        CREATE INDEX IF NOT EXISTS idx_programs_channel ON programs(channel_id);",
    )?;
    Ok(conn)
}

pub fn get_visible_channels(conn: &Connection) -> Result<Vec<Channel>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, icon_url, visible, sort_order FROM channels WHERE visible = 1 ORDER BY sort_order, name",
    )?;
    let channels = stmt
        .query_map([], |row| {
            Ok(Channel {
                id: row.get(0)?,
                name: row.get(1)?,
                icon_url: row.get(2)?,
                visible: row.get::<_, i32>(3)? != 0,
                sort_order: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(channels)
}

pub fn get_all_channels(conn: &Connection) -> Result<Vec<Channel>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, icon_url, visible, sort_order FROM channels ORDER BY sort_order, name",
    )?;
    let channels = stmt
        .query_map([], |row| {
            Ok(Channel {
                id: row.get(0)?,
                name: row.get(1)?,
                icon_url: row.get(2)?,
                visible: row.get::<_, i32>(3)? != 0,
                sort_order: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(channels)
}

pub fn get_programs_in_range(conn: &Connection, from: i64, to: i64) -> Result<Vec<Program>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.channel_id, p.title, p.description, p.category, p.start_time, p.end_time
         FROM programs p
         JOIN channels c ON p.channel_id = c.id
         WHERE c.visible = 1 AND p.end_time > ?1 AND p.start_time < ?2
         ORDER BY p.start_time",
    )?;
    let programs = stmt
        .query_map([from, to], |row| {
            Ok(Program {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                start_time: row.get(5)?,
                end_time: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(programs)
}

pub fn get_favourite_programs(conn: &Connection, from: i64, to: i64) -> Result<Vec<Program>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.channel_id, p.title, p.description, p.category, p.start_time, p.end_time
         FROM programs p
         JOIN favourites f ON p.title = f.title
         WHERE p.end_time > ?1 AND p.start_time < ?2
         ORDER BY p.start_time",
    )?;
    let programs = stmt
        .query_map([from, to], |row| {
            Ok(Program {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                start_time: row.get(5)?,
                end_time: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(programs)
}

pub fn get_favourites(conn: &Connection) -> Result<Vec<Favourite>, AppError> {
    let mut stmt = conn.prepare("SELECT title, added_at FROM favourites ORDER BY added_at DESC")?;
    let favs = stmt
        .query_map([], |row| Ok(Favourite { title: row.get(0)?, added_at: row.get(1)? }))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(favs)
}

pub fn toggle_favourite(conn: &Connection, title: &str) -> Result<bool, AppError> {
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM favourites WHERE title = ?1",
        [title],
        |row| row.get(0),
    )?;
    if exists {
        conn.execute("DELETE FROM favourites WHERE title = ?1", [title])?;
        Ok(false)
    } else {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO favourites (title, added_at) VALUES (?1, ?2)",
            rusqlite::params![title, now],
        )?;
        Ok(true)
    }
}

pub fn set_channel_visibility(conn: &Connection, channel_id: &str, visible: bool) -> Result<(), AppError> {
    conn.execute(
        "UPDATE channels SET visible = ?1 WHERE id = ?2",
        rusqlite::params![visible as i32, channel_id],
    )?;
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    match conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| row.get(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

pub fn upsert_channels(conn: &Connection, channels: &[Channel]) -> Result<(), AppError> {
    let mut stmt = conn.prepare(
        "INSERT INTO channels (id, name, icon_url, visible, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, icon_url = excluded.icon_url, sort_order = excluded.sort_order",
    )?;
    for ch in channels {
        stmt.execute(rusqlite::params![
            ch.id, ch.name, ch.icon_url, ch.visible as i32, ch.sort_order
        ])?;
    }
    Ok(())
}

pub fn upsert_programs(conn: &Connection, programs: &[Program]) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp();
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO programs (id, channel_id, title, description, category, start_time, end_time, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;
    for p in programs {
        stmt.execute(rusqlite::params![
            p.id, p.channel_id, p.title, p.description, p.category, p.start_time, p.end_time, now
        ])?;
    }
    Ok(())
}

pub fn get_cache_age_seconds(conn: &Connection) -> Result<Option<i64>, AppError> {
    let result = conn.query_row(
        "SELECT MIN(fetched_at) FROM programs WHERE end_time > ?1",
        [chrono::Utc::now().timestamp()],
        |row| row.get::<_, Option<i64>>(0),
    );
    match result {
        Ok(Some(oldest)) => Ok(Some(chrono::Utc::now().timestamp() - oldest)),
        Ok(None) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
