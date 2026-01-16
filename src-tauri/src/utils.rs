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

#[cfg(test)]
mod tests {
    use super::is_valid_time;

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
}
