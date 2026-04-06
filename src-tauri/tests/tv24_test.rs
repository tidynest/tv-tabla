use tv_tabla_lib::tv24;

const CHANNELS_HTML: &str = r#"
<div class="page"><div class="settings"><ul id="addremove-channels">
  <li data-channel="69430" data-channelselected="yes">
    <img src="https://assets.tv24.co/channels/svt1-l.webp" width="40" height="30" alt="Logo: SVT1" title="SVT1">
    <div class="info"><h3>SVT1</h3></div>
  </li>
  <li data-channel="69431" data-channelselected="yes">
    <img src="https://assets.tv24.co/channels/svt2-l.webp" width="40" height="30" alt="Logo: SVT2" title="SVT2">
    <div class="info"><h3>SVT2</h3></div>
  </li>
  <li data-channel="54090" data-channelselected="no">
    <img src="https://assets.tv24.co/channels/tv4-l.webp" width="40" height="30" alt="Logo: TV4" title="TV4">
    <div class="info"><h3>TV4</h3></div>
  </li>
</ul></div></div>
"#;

const PROGRAMS_HTML: &str = r#"
<ul class="section-items with-ended">
  <li class="ended"><a href="/b/abc123" class="program">
    <span class="time">18:00</span>
    <div class="meta">
      <h3>Husdrömmar</h3>
      <span class="desc">Säsong 13 Avsnitt 11</span>
      <p>Systrarna bygger stugor i Roslagen.</p>
    </div>
  </a></li>
  <li><a href="/b/def456" class="program">
    <span class="time">19:00</span>
    <div class="meta">
      <h3>Rapport</h3>
      <p>Nyheter.</p>
    </div>
  </a></li>
  <li><a href="/b/ghi789" class="program">
    <span class="time">19:30</span>
    <div class="meta">
      <h3>Kvällsfilm</h3>
      <span class="desc">Drama (2024)</span>
      <p>En spännande film.</p>
    </div>
  </a></li>
</ul>
"#;

const OVERNIGHT_HTML: &str = r#"
<ul class="section-items">
  <li><a href="/b/late1" class="program">
    <span class="time">22:00</span>
    <div class="meta"><h3>Kvällsnytt</h3><p>Sena nyheter.</p></div>
  </a></li>
  <li><a href="/b/late2" class="program">
    <span class="time">23:30</span>
    <div class="meta"><h3>Nattfilm</h3><p>Film.</p></div>
  </a></li>
  <li><a href="/b/early1" class="program">
    <span class="time">02:00</span>
    <div class="meta"><h3>Repriser</h3><p>Repriser.</p></div>
  </a></li>
</ul>
"#;

#[test]
fn test_parse_channels() {
    let channels = tv24::parse_channels(CHANNELS_HTML).unwrap();
    assert_eq!(channels.len(), 3);

    assert_eq!(channels[0].id, "svt1");
    assert_eq!(channels[0].name, "SVT1");
    assert!(channels[0].icon_url.as_ref().unwrap().contains("svt1"));
    assert_eq!(channels[0].sort_order, 0);

    assert_eq!(channels[1].id, "svt2");
    assert_eq!(channels[2].id, "tv4");
    assert_eq!(channels[2].sort_order, 2);
}

#[test]
fn test_parse_programs() {
    let programs = tv24::parse_programs(PROGRAMS_HTML, "svt1", "2026-04-06").unwrap();
    assert_eq!(programs.len(), 3);

    assert_eq!(programs[0].title, "Husdrömmar");
    assert_eq!(programs[0].channel_id, "svt1");
    assert_eq!(programs[0].description.as_deref(), Some("Systrarna bygger stugor i Roslagen."));
    assert_eq!(programs[0].category.as_deref(), Some("Säsong 13 Avsnitt 11"));
    assert_eq!(programs[0].id, "abc123");

    // End time of first = start time of second
    assert_eq!(programs[0].end_time, programs[1].start_time);

    assert_eq!(programs[1].title, "Rapport");
    assert!(programs[1].category.is_none());

    assert_eq!(programs[2].title, "Kvällsfilm");
    assert_eq!(programs[2].category.as_deref(), Some("Drama (2024)"));
    // Last program gets +3600 default
    assert_eq!(programs[2].end_time, programs[2].start_time + 3600);
}

#[test]
fn test_overnight_rollover() {
    let programs = tv24::parse_programs(OVERNIGHT_HTML, "svt1", "2026-04-06").unwrap();
    assert_eq!(programs.len(), 3);

    // 02:00 should be on the next day (later than 23:30)
    assert!(programs[2].start_time > programs[1].start_time);
    // Specifically 02:00 on 2026-04-07 vs 23:30 on 2026-04-06
    let gap = programs[2].start_time - programs[1].start_time;
    assert!(gap > 0, "overnight program must be after previous: gap={gap}");
    // ~2.5 hours gap (23:30 to 02:00)
    assert!((gap - 9000).abs() < 60, "expected ~2.5h gap, got {gap}s");
}

#[test]
fn test_parse_empty_html() {
    let programs = tv24::parse_programs("<div></div>", "svt1", "2026-04-06").unwrap();
    assert!(programs.is_empty());

    let channels = tv24::parse_channels("<div></div>").unwrap();
    assert!(channels.is_empty());
}
