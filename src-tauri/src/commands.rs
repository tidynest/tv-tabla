use crate::error::AppError;
use crate::models::{Channel, Favourite, Program};
use crate::AppState;

#[tauri::command]
pub async fn get_channels(state: tauri::State<'_, AppState>) -> Result<Vec<Channel>, AppError> {
    let db = state.db.lock().unwrap();
    crate::db::get_visible_channels(&db)
}

#[tauri::command]
pub async fn get_programs(
    state: tauri::State<'_, AppState>,
    from: i64,
    to: i64,
) -> Result<Vec<Program>, AppError> {
    let db = state.db.lock().unwrap();
    crate::db::get_programs_in_range(&db, from, to)
}

#[tauri::command]
pub async fn get_favourites(state: tauri::State<'_, AppState>) -> Result<Vec<Favourite>, AppError> {
    let db = state.db.lock().unwrap();
    crate::db::get_favourites(&db)
}

#[tauri::command]
pub async fn toggle_favourite(
    state: tauri::State<'_, AppState>,
    title: String,
) -> Result<bool, AppError> {
    let db = state.db.lock().unwrap();
    crate::db::toggle_favourite(&db, &title)
}

#[tauri::command]
pub async fn set_channel_visibility(
    state: tauri::State<'_, AppState>,
    channel_id: String,
    visible: bool,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    crate::db::set_channel_visibility(&db, &channel_id, visible)
}

#[tauri::command]
pub async fn get_setting(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<Option<String>, AppError> {
    let db = state.db.lock().unwrap();
    crate::db::get_setting(&db, &key)
}

#[tauri::command]
pub async fn set_setting(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    crate::db::set_setting(&db, &key, &value)
}

#[tauri::command]
pub async fn get_cache_age(state: tauri::State<'_, AppState>) -> Result<Option<i64>, AppError> {
    let db = state.db.lock().unwrap();
    crate::db::get_cache_age_seconds(&db)
}

#[tauri::command]
pub async fn get_all_channels(state: tauri::State<'_, AppState>) -> Result<Vec<Channel>, AppError> {
    let db = state.db.lock().unwrap();
    crate::db::get_all_channels(&db)
}

#[tauri::command]
pub async fn get_favourite_programs(
    state: tauri::State<'_, AppState>,
    from: i64,
    to: i64,
) -> Result<Vec<Program>, AppError> {
    let db = state.db.lock().unwrap();
    crate::db::get_favourite_programs(&db, from, to)
}

#[tauri::command]
pub async fn refresh_data(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let channels = crate::fetcher::fetch_channels().await?;

    let visible = {
        let db = state.db.lock().unwrap();
        crate::db::upsert_channels(&db, &channels)?;
        crate::db::get_visible_channels(&db)?
    };

    let today = chrono::Local::now().date_naive();
    let dates: Vec<String> = (0..7)
        .map(|d| {
            (today + chrono::Duration::days(d))
                .format("%Y-%m-%d")
                .to_string()
        })
        .collect();

    for date in &dates {
        for channel in &visible {
            match crate::fetcher::fetch_programs(&channel.id, date).await {
                Ok(programs) => {
                    let db = state.db.lock().unwrap();
                    crate::db::upsert_programs(&db, &programs)?;
                }
                Err(e) => {
                    log::warn!("Failed to fetch {} on {}: {}", channel.id, date, e);
                }
            }
        }
    }
    Ok(())
}
