use crate::error::AppError;
use crate::models::{Channel, Program};

pub async fn fetch_channels() -> Result<Vec<Channel>, AppError> {
    let bytes = fetch_gzipped("https://xmltv.xmltv.se/channels-Sweden.xml.gz").await?;
    let xml = decode_xmltv_bytes(&bytes)?;
    crate::xmltv::parse_channels(&xml)
}

pub async fn fetch_programs(channel_id: &str, date: &str) -> Result<Vec<Program>, AppError> {
    let url = format!("https://xmltv.xmltv.se/{}_{}.xml.gz", channel_id, date);
    let bytes = fetch_gzipped(&url).await?;
    let xml = decode_xmltv_bytes(&bytes)?;
    crate::xmltv::parse_programs(&xml)
}

async fn fetch_gzipped(url: &str) -> Result<Vec<u8>, AppError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "TV-Tabla/0.1.0 (desktop app)")
        .send()
        .await?
        .bytes()
        .await?;
    Ok(resp.to_vec())
}

fn decode_xmltv_bytes(bytes: &[u8]) -> Result<String, AppError> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(bytes);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(&decompressed);
    Ok(cow.into_owned().into())
}
