use crate::error::AppError;
use crate::models::{Channel, Program};

const BASE: &str = "https://tv24.se";
const USER_AGENT: &str = "TV-Tabla/0.1.0 (desktop app)";

pub async fn fetch_channels() -> Result<Vec<Channel>, AppError> {
    let html = fetch_text(&format!("{BASE}/x/settings/addremove")).await?;
    crate::tv24::parse_channels(&html)
}

pub async fn fetch_programs(channel_slug: &str, date: &str) -> Result<Vec<Program>, AppError> {
    let url = format!("{BASE}/x/channel/{channel_slug}/0/{date}");
    let html = fetch_text(&url).await?;
    crate::tv24::parse_programs(&html, channel_slug, date)
}

async fn fetch_text(url: &str) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    let text = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .text()
        .await?;
    Ok(text)
}
