use anyhow::Result;
use novel_shared::FrontMatter;

pub(crate) fn validate_frontmatter_dates(frontmatter: &FrontMatter, source: &str) -> Result<()> {
    validate_date_field("published_at", frontmatter.published_at.as_deref(), source)?;
    validate_date_field("updated_at", frontmatter.updated_at.as_deref(), source)?;
    validate_date_field("expires_at", frontmatter.expires_at.as_deref(), source)?;
    Ok(())
}

fn validate_date_field(field: &str, value: Option<&str>, source: &str) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if is_valid_date_or_rfc3339(value) {
        Ok(())
    } else {
        anyhow::bail!(
            "Invalid frontmatter date `{}` in {}: `{}`. Expected YYYY-MM-DD or RFC3339.",
            field,
            source,
            value
        )
    }
}

fn is_valid_date_or_rfc3339(value: &str) -> bool {
    is_valid_ymd(value) || is_valid_rfc3339(value)
}

fn is_valid_ymd(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(idx, b)| idx == 4 || idx == 7 || b.is_ascii_digit())
    {
        return false;
    }

    let year = parse_u32(&value[0..4]);
    let month = parse_u32(&value[5..7]);
    let day = parse_u32(&value[8..10]);
    match (year, month, day) {
        (Some(year), Some(month @ 1..=12), Some(day)) => {
            day >= 1 && day <= days_in_month(year, month)
        }
        _ => false,
    }
}

fn is_valid_rfc3339(value: &str) -> bool {
    let Some((date, time_and_tz)) = value.split_once('T') else {
        return false;
    };
    if !is_valid_ymd(date) {
        return false;
    }

    let (time, tz) = split_time_zone(time_and_tz);
    is_valid_time(time) && is_valid_timezone(tz)
}

fn split_time_zone(value: &str) -> (&str, &str) {
    if let Some(time) = value.strip_suffix('Z') {
        return (time, "Z");
    }

    let offset_idx = value
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (idx > 0 && matches!(ch, '+' | '-')).then_some(idx));

    if let Some(idx) = offset_idx {
        (&value[..idx], &value[idx..])
    } else {
        (value, "")
    }
}

fn is_valid_time(value: &str) -> bool {
    let (hms, fraction) = value.split_once('.').unwrap_or((value, ""));
    if !fraction.is_empty() && !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }

    let parts: Vec<&str> = hms.split(':').collect();
    if parts.len() != 3 {
        return false;
    }

    let hour = parse_u32(parts[0]);
    let minute = parse_u32(parts[1]);
    let second = parse_u32(parts[2]);
    matches!(
        (hour, minute, second),
        (Some(0..=23), Some(0..=59), Some(0..=60))
    )
}

fn is_valid_timezone(value: &str) -> bool {
    if value == "Z" {
        return true;
    }

    let Some(sign) = value.as_bytes().first() else {
        return false;
    };
    if !matches!(sign, b'+' | b'-') {
        return false;
    }

    let offset = &value[1..];
    let parts: Vec<&str> = offset.split(':').collect();
    if parts.len() != 2 {
        return false;
    }

    matches!(
        (parse_u32(parts[0]), parse_u32(parts[1])),
        (Some(0..=23), Some(0..=59))
    )
}

fn parse_u32(value: &str) -> Option<u32> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::is_valid_date_or_rfc3339;

    #[test]
    fn accepts_calendar_dates_and_rfc3339() {
        assert!(is_valid_date_or_rfc3339("2026-07-07"));
        assert!(is_valid_date_or_rfc3339("2024-02-29"));
        assert!(is_valid_date_or_rfc3339("2026-07-07T12:34:56Z"));
        assert!(is_valid_date_or_rfc3339("2026-07-07T12:34:56.789+08:00"));
    }

    #[test]
    fn rejects_invalid_dates() {
        assert!(!is_valid_date_or_rfc3339("2026-02-29"));
        assert!(!is_valid_date_or_rfc3339("2026-13-01"));
        assert!(!is_valid_date_or_rfc3339("2026-07-07 12:34:56"));
        assert!(!is_valid_date_or_rfc3339("2026-07-07T25:34:56Z"));
    }
}
