use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::{Channel, Program};

/// Parse "YYYYMMDDhhmmss +HHMM" into a UTC Unix timestamp.
pub fn parse_xmltv_timestamp(ts: &str) -> Result<i64, AppError> {
    use chrono::{FixedOffset, TimeZone};

    let ts = ts.trim();
    // Minimum: 14-char datetime + space + offset (e.g. "+0200")
    if ts.len() < 20 {
        return Err(AppError::Custom(format!("timestamp too short: {ts}")));
    }

    let (dt_part, tz_part) = ts.split_at(14);
    let tz_str = tz_part.trim();

    let sign: i32 = if tz_str.starts_with('-') { -1 } else { 1 };
    let tz_digits: &str = tz_str.trim_start_matches(['+', '-']);
    if tz_digits.len() != 4 {
        return Err(AppError::Custom(format!("invalid timezone: {tz_str}")));
    }
    let tz_hours: i32 = tz_digits[..2].parse().map_err(|_| AppError::Custom(format!("invalid tz hours: {tz_str}")))?;
    let tz_mins: i32  = tz_digits[2..].parse().map_err(|_| AppError::Custom(format!("invalid tz mins: {tz_str}")))?;
    let offset_secs = sign * (tz_hours * 3600 + tz_mins * 60);

    let year:  i32 = dt_part[0..4].parse().map_err(|_| AppError::Custom(format!("bad year in {dt_part}")))?;
    let month: u32 = dt_part[4..6].parse().map_err(|_| AppError::Custom(format!("bad month in {dt_part}")))?;
    let day:   u32 = dt_part[6..8].parse().map_err(|_| AppError::Custom(format!("bad day in {dt_part}")))?;
    let hour:  u32 = dt_part[8..10].parse().map_err(|_| AppError::Custom(format!("bad hour in {dt_part}")))?;
    let min:   u32 = dt_part[10..12].parse().map_err(|_| AppError::Custom(format!("bad min in {dt_part}")))?;
    let sec:   u32 = dt_part[12..14].parse().map_err(|_| AppError::Custom(format!("bad sec in {dt_part}")))?;

    let offset = FixedOffset::east_opt(offset_secs)
        .ok_or_else(|| AppError::Custom(format!("out-of-range offset: {offset_secs}")))?;
    let dt = offset
        .with_ymd_and_hms(year, month, day, hour, min, sec)
        .single()
        .ok_or_else(|| AppError::Custom(format!("invalid datetime: {ts}")))?;

    Ok(dt.timestamp())
}

pub fn parse_channels(xml: &str) -> Result<Vec<Channel>, AppError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut channels: Vec<Channel> = Vec::new();
    let mut current: Option<PartialChannel> = None;
    let mut in_display_name = false;
    let mut sort_order: i32 = 0;

    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"channel" => {
                let id = attr_value(&e, b"id")?;
                current = Some(PartialChannel { id, name: String::new(), icon_url: None });
            }
            Event::Empty(e) if e.name().as_ref() == b"icon" => {
                if let Some(ref mut ch) = current {
                    ch.icon_url = attr_value(&e, b"src").ok();
                }
            }
            Event::Start(e) if e.name().as_ref() == b"display-name" => {
                in_display_name = current.is_some();
            }
            Event::Text(e) if in_display_name => {
                if let Some(ref mut ch) = current {
                    ch.name = e.xml_content().map_err(quick_xml::Error::from)?.into_owned();
                }
            }
            Event::End(e) if e.name().as_ref() == b"display-name" => {
                in_display_name = false;
            }
            Event::End(e) if e.name().as_ref() == b"channel" => {
                if let Some(ch) = current.take() {
                    channels.push(Channel {
                        id: ch.id,
                        name: ch.name,
                        icon_url: ch.icon_url,
                        visible: true,
                        sort_order,
                    });
                    sort_order += 1;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(channels)
}

pub fn parse_programs(xml: &str) -> Result<Vec<Program>, AppError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut programs: Vec<Program> = Vec::new();
    let mut current: Option<PartialProgram> = None;
    let mut capture: Option<ProgramField> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"programme" => {
                let channel_id = attr_value(&e, b"channel")?;
                let start_ts   = attr_value(&e, b"start")?;
                let stop_ts    = attr_value(&e, b"stop").ok();

                let start_time = parse_xmltv_timestamp(&start_ts)?;
                let end_time   = match stop_ts {
                    Some(ref s) => parse_xmltv_timestamp(s)?,
                    None => start_time + 3600,
                };

                let id = format!("{channel_id}_{start_time}");
                current = Some(PartialProgram {
                    id,
                    channel_id,
                    start_time,
                    end_time,
                    title: String::new(),
                    description: None,
                    category: None,
                });
            }
            Event::Start(e) if current.is_some() => {
                capture = match e.name().as_ref() {
                    b"title"    => Some(ProgramField::Title),
                    b"desc"     => Some(ProgramField::Desc),
                    b"category" => Some(ProgramField::Category),
                    _ => None,
                };
            }
            Event::Text(e) => {
                if let (Some(ref mut prog), Some(ref field)) = (current.as_mut(), capture.as_ref()) {
                    let text = e.xml_content().map_err(quick_xml::Error::from)?.into_owned();
                    match field {
                        ProgramField::Title    => prog.title = text,
                        ProgramField::Desc     => prog.description = Some(text),
                        ProgramField::Category => prog.category = Some(text),
                    }
                }
            }
            Event::End(e) => {
                match e.name().as_ref() {
                    b"title" | b"desc" | b"category" => capture = None,
                    b"programme" => {
                        if let Some(p) = current.take() {
                            programs.push(Program {
                                id: p.id,
                                channel_id: p.channel_id,
                                title: p.title,
                                description: p.description,
                                category: p.category,
                                start_time: p.start_time,
                                end_time: p.end_time,
                            });
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(programs)
}

// --- helpers ---

fn attr_value(e: &quick_xml::events::BytesStart, name: &[u8]) -> Result<String, AppError> {
    for attr in e.attributes() {
        let a = attr.map_err(quick_xml::Error::InvalidAttr)?;
        if a.key.as_ref() == name {
            return Ok(String::from_utf8_lossy(&a.value).into_owned());
        }
    }
    Err(AppError::Custom(format!(
        "missing attribute '{}' on <{}>",
        String::from_utf8_lossy(name),
        String::from_utf8_lossy(e.name().as_ref())
    )))
}

struct PartialChannel {
    id: String,
    name: String,
    icon_url: Option<String>,
}

struct PartialProgram {
    id: String,
    channel_id: String,
    start_time: i64,
    end_time: i64,
    title: String,
    description: Option<String>,
    category: Option<String>,
}

enum ProgramField {
    Title,
    Desc,
    Category,
}
