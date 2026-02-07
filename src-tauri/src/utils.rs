pub fn is_valid_time(time: &str) -> bool {
    if time.len() != 5 {
        return false;
    }

    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return false;
    }

    let hour: u8 = match parts[0].parse() {
        Ok(h) if h < 24 => h,
        _ => return false,
    };

    let minute: u8 = match parts[1].parse() {
        Ok(m) if m < 60 => m,
        _ => return false,
    };

    let _ = (hour, minute);
    true
}

pub fn parse_datetime_to_rfc3339(value: &str) -> Option<String> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(value) {
        return Some(dt.to_rfc3339());
    }

    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
        if let Some(local) = chrono::Local.from_local_datetime(&naive).single() {
            return Some(local.to_rfc3339());
        }
    }

    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        if let Some(local) = chrono::Local.from_local_datetime(&naive).single() {
            return Some(local.to_rfc3339());
        }
    }
    }

    if let Ok(date) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let naive = date.and_hms_opt(0, 0, 0)?;
        let dt = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive, chrono::Utc);
        return Some(dt.to_rfc3339());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{is_valid_time, parse_datetime_to_rfc3339};

    #[test]
    fn rejects_invalid_times() {
        assert!(!is_valid_time(""));
        assert!(!is_valid_time("1:00"));
        assert!(!is_valid_time("01:0"));
        assert!(!is_valid_time("24:00"));
        assert!(!is_valid_time("23:60"));
        assert!(!is_valid_time("aa:bb"));
        assert!(!is_valid_time("12-34"));
    }

    #[test]
    fn accepts_valid_times() {
        assert!(is_valid_time("00:00"));
        assert!(is_valid_time("09:05"));
        assert!(is_valid_time("23:59"));
    }

    #[test]
    fn parses_rfc3339_and_naive_datetime() {
        let parsed = parse_datetime_to_rfc3339("2026-02-07T09:30:00").unwrap();
        assert!(parsed.starts_with("2026-02-07T09:30:00"));
    }
}
