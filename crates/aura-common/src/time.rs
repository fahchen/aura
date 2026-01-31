//! Time parsing utilities.

use chrono::{DateTime, Utc};
use std::time::{Duration, SystemTime};

/// Parse an RFC 3339 timestamp string into `SystemTime`.
///
/// Examples:
/// - "2026-01-31T12:45:31.053Z"
/// - "2026-01-31T12:45:31Z"
/// - "2026-01-31T12:45:31.053+00:00"
pub fn parse_rfc3339_system_time(ts: &str) -> Option<SystemTime> {
    let parsed = DateTime::parse_from_rfc3339(ts).ok()?;
    let utc = parsed.with_timezone(&Utc);
    let secs = utc.timestamp();
    if secs < 0 {
        return None;
    }
    let nanos = utc.timestamp_subsec_nanos() as u64;
    let base = SystemTime::UNIX_EPOCH + Duration::from_secs(secs as u64);
    Some(base + Duration::from_nanos(nanos))
}

#[cfg(test)]
mod tests {
    use super::parse_rfc3339_system_time;
    use std::time::{Duration, SystemTime};

    #[test]
    fn parse_rfc3339_with_z_suffix() {
        let ts = "2026-01-31T12:45:31.053Z";
        let result = parse_rfc3339_system_time(ts);
        assert!(result.is_some());
        assert!(result.unwrap() > SystemTime::UNIX_EPOCH);
    }

    #[test]
    fn parse_rfc3339_without_millis() {
        let ts = "2026-01-31T12:45:31Z";
        let result = parse_rfc3339_system_time(ts);
        assert!(result.is_some());
    }

    #[test]
    fn parse_rfc3339_with_offset() {
        let ts = "2026-01-31T12:45:31.053+00:00";
        let result = parse_rfc3339_system_time(ts);
        assert!(result.is_some());
    }

    #[test]
    fn parse_rfc3339_with_negative_offset() {
        let ts = "2026-01-31T07:45:31.053-05:00";
        let result = parse_rfc3339_system_time(ts);
        assert!(result.is_some());
    }

    #[test]
    fn parse_rfc3339_invalid() {
        assert!(parse_rfc3339_system_time("not a timestamp").is_none());
        assert!(parse_rfc3339_system_time("2026-01-31").is_none());
        assert!(parse_rfc3339_system_time("12:45:31").is_none());
        assert!(parse_rfc3339_system_time("").is_none());
    }

    #[test]
    fn parse_rfc3339_is_recent() {
        let now = chrono::Utc::now();
        let ts = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let parsed = parse_rfc3339_system_time(&ts).unwrap();
        let elapsed = SystemTime::now()
            .duration_since(parsed)
            .unwrap_or(Duration::ZERO);
        assert!(elapsed.as_secs() < 5, "Parsed time should be recent");
    }
}
