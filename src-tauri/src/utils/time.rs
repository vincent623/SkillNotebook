use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub fn now_iso() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "2026-04-13T00:00:00Z".to_string())
}

pub fn today_slug() -> String {
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}{:02}{:02}",
        now.year(),
        u8::from(now.month()),
        now.day()
    )
}
