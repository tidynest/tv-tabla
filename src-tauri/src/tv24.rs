use scraper::{Html, Selector};

use crate::error::AppError;
use crate::models::{Channel, Program};

/// Parse the channel list from tv24.se/x/settings/addremove HTML.
/// Extracts slug (used as channel ID), display name, and icon URL.
pub fn parse_channels(html: &str) -> Result<Vec<Channel>, AppError> {
    let doc = Html::parse_fragment(html);
    let li_sel = Selector::parse("li[data-channel]").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let img_sel = Selector::parse("img[src*='assets.tv24']").unwrap();

    let mut channels = Vec::new();
    let mut sort_order: i32 = 0;

    for li in doc.select(&li_sel) {
        let name = match li.select(&h3_sel).next() {
            Some(el) => el.text().collect::<String>().trim().to_string(),
            None => continue,
        };

        let (slug, icon_url) = match li.select(&img_sel).next() {
            Some(img) => {
                let src = img.value().attr("src").unwrap_or_default();
                let slug = extract_slug(src);
                (slug, Some(src.to_string()))
            }
            None => continue,
        };

        if slug.is_empty() {
            continue;
        }

        channels.push(Channel {
            id: slug,
            name,
            icon_url,
            visible: true,
            sort_order,
        });
        sort_order += 1;
    }

    Ok(channels)
}

/// Parse the program schedule from tv24.se/x/channel/{slug}/0/{date} HTML.
/// `date` must be "YYYY-MM-DD" — used to build absolute timestamps.
pub fn parse_programs(html: &str, channel_id: &str, date: &str) -> Result<Vec<Program>, AppError> {
    let doc = Html::parse_fragment(html);
    let li_sel = Selector::parse("li").unwrap();
    let time_sel = Selector::parse(".time").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let desc_sel = Selector::parse(".desc").unwrap();
    let p_sel = Selector::parse("p").unwrap();
    let a_sel = Selector::parse("a").unwrap();

    let base_date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| AppError::Custom(format!("bad date '{date}': {e}")))?;

    struct RawProgram {
        href: String,
        title: String,
        description: Option<String>,
        category: Option<String>,
        hour: u32,
        minute: u32,
    }

    let mut raw: Vec<RawProgram> = Vec::new();

    for li in doc.select(&li_sel) {
        let time_text = match li.select(&time_sel).next() {
            Some(el) => el.text().collect::<String>(),
            None => continue,
        };

        let (hour, minute) = match parse_hhmm(&time_text) {
            Some(hm) => hm,
            None => continue,
        };

        let title = match li.select(&h3_sel).next() {
            Some(el) => el.text().collect::<String>().trim().to_string(),
            None => continue,
        };

        if title.is_empty() {
            continue;
        }

        let href = li
            .select(&a_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or_default()
            .to_string();

        let category = li
            .select(&desc_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty());

        let description = li
            .select(&p_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty());

        raw.push(RawProgram {
            href,
            title,
            description,
            category,
            hour,
            minute,
        });
    }

    // Convert to Programs with absolute timestamps.
    // Detect day rollover: when time goes backwards, it's the next day.
    let mut programs = Vec::new();
    let mut prev_minutes: Option<u32> = None;
    let mut day_offset: i64 = 0;

    for (i, r) in raw.iter().enumerate() {
        let total_min = r.hour * 60 + r.minute;
        if let Some(prev) = prev_minutes {
            if total_min < prev {
                day_offset = 1;
            }
        }
        prev_minutes = Some(total_min);

        let actual_date = base_date + chrono::Duration::days(day_offset);
        let start_time = date_hm_to_timestamp(actual_date, r.hour, r.minute)?;

        // End time = next program's start, or +60min for the last one
        let end_time = if i + 1 < raw.len() {
            let next = &raw[i + 1];
            let next_min = next.hour * 60 + next.minute;
            let next_day = if next_min < total_min {
                day_offset + 1
            } else {
                day_offset
            };
            let next_date = base_date + chrono::Duration::days(next_day);
            date_hm_to_timestamp(next_date, next.hour, next.minute)?
        } else {
            start_time + 3600
        };

        let id = if r.href.is_empty() {
            format!("{channel_id}_{start_time}")
        } else {
            r.href.trim_start_matches("/b/").to_string()
        };

        programs.push(Program {
            id,
            channel_id: channel_id.to_string(),
            title: r.title.clone(),
            description: r.description.clone(),
            category: r.category.clone(),
            start_time,
            end_time,
        });
    }

    Ok(programs)
}

/// Extract channel slug from icon URL like "https://assets.tv24.co/channels/svt1-l.webp"
fn extract_slug(url: &str) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or_default()
        .strip_suffix("-l.webp")
        .or_else(|| url.rsplit('/').next().unwrap_or_default().strip_suffix(".webp"))
        .unwrap_or_default()
        .to_string()
}

/// Parse "HH:MM" into (hour, minute).
fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let s = s.trim();
    let (h, m) = s.split_once(':')?;
    Some((h.parse().ok()?, m.parse().ok()?))
}

/// Convert a date + hour/minute in Swedish local time to a UTC Unix timestamp.
fn date_hm_to_timestamp(date: chrono::NaiveDate, hour: u32, minute: u32) -> Result<i64, AppError> {
    let time = chrono::NaiveTime::from_hms_opt(hour, minute, 0)
        .ok_or_else(|| AppError::Custom(format!("invalid time {hour}:{minute}")))?;
    let naive = chrono::NaiveDateTime::new(date, time);
    let local = chrono::Local
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| AppError::Custom(format!("ambiguous local time: {naive}")))?;
    Ok(local.timestamp())
}

use chrono::TimeZone;
