use tv_tabla_lib::xmltv::{parse_channels, parse_programs, parse_xmltv_timestamp};

const SAMPLE_CHANNELS_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<tv>
  <channel id="svt1.svt.se">
    <display-name>SVT 1</display-name>
    <base-url>https://xmltv.xmltv.se/</base-url>
    <icon src="https://xmltv.xmltv.se/logos/svt1.svt.se.png"/>
  </channel>
  <channel id="tv4.se">
    <display-name>TV4</display-name>
    <base-url>https://xmltv.xmltv.se/</base-url>
  </channel>
</tv>"#;

const SAMPLE_PROGRAMS_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<tv source-info-name="xmltv.se">
  <programme start="20260406190000 +0200" stop="20260406200000 +0200" channel="svt1.svt.se">
    <title lang="sv">Aktuellt</title>
    <desc lang="sv">Nyheter och reportage.</desc>
    <category lang="sv">Nyheter</category>
  </programme>
  <programme start="20260406200000 +0200" stop="20260406210000 +0200" channel="svt1.svt.se">
    <title lang="sv">Rapport</title>
    <sub-title lang="sv">Kvallens nyheter</sub-title>
    <desc lang="sv">Nyheter fran Sverige och varlden.</desc>
    <category lang="sv">Nyheter</category>
  </programme>
  <programme start="20260406210000 +0200" channel="svt1.svt.se">
    <title lang="sv">No Stop Time Show</title>
  </programme>
</tv>"#;

#[test]
fn test_parse_channels() {
    let channels = parse_channels(SAMPLE_CHANNELS_XML).expect("parse failed");
    assert_eq!(channels.len(), 2);

    let svt1 = &channels[0];
    assert_eq!(svt1.id, "svt1.svt.se");
    assert_eq!(svt1.name, "SVT 1");
    assert_eq!(svt1.icon_url.as_deref(), Some("https://xmltv.xmltv.se/logos/svt1.svt.se.png"));

    let tv4 = &channels[1];
    assert_eq!(tv4.id, "tv4.se");
    assert_eq!(tv4.name, "TV4");
    assert!(tv4.icon_url.is_none(), "tv4 has no icon");
}

#[test]
fn test_parse_programs() {
    let programs = parse_programs(SAMPLE_PROGRAMS_XML).expect("parse failed");
    assert_eq!(programs.len(), 3);

    let aktuellt = &programs[0];
    assert_eq!(aktuellt.title, "Aktuellt");
    assert_eq!(aktuellt.channel_id, "svt1.svt.se");
    assert_eq!(aktuellt.description.as_deref(), Some("Nyheter och reportage."));
    assert_eq!(aktuellt.category.as_deref(), Some("Nyheter"));
    // 2026-04-06T19:00:00+0200 = 2026-04-06T17:00:00Z
    assert_eq!(aktuellt.start_time, 1775494800);
    // 2026-04-06T20:00:00+0200 = 2026-04-06T18:00:00Z
    assert_eq!(aktuellt.end_time, 1775498400);

    let rapport = &programs[1];
    assert_eq!(rapport.title, "Rapport");
    assert_eq!(rapport.description.as_deref(), Some("Nyheter fran Sverige och varlden."));
    assert_eq!(rapport.category.as_deref(), Some("Nyheter"));
}

#[test]
fn test_parse_program_missing_stop_time() {
    let programs = parse_programs(SAMPLE_PROGRAMS_XML).expect("parse failed");
    let no_stop = &programs[2];
    assert_eq!(no_stop.title, "No Stop Time Show");
    assert_eq!(no_stop.end_time, no_stop.start_time + 3600, "missing stop should default to +1h");
}

#[test]
fn test_parse_timestamp() {
    // 2026-04-06T19:00:00+02:00 = 2026-04-06T17:00:00Z
    let ts = parse_xmltv_timestamp("20260406190000 +0200").expect("parse failed");
    assert_eq!(ts, 1775494800);
}

#[test]
fn test_parse_empty_xml() {
    let channels = parse_channels("<tv></tv>").expect("channels parse failed");
    assert!(channels.is_empty(), "empty tv element should yield no channels");

    let programs = parse_programs("<tv></tv>").expect("programs parse failed");
    assert!(programs.is_empty(), "empty tv element should yield no programs");
}
