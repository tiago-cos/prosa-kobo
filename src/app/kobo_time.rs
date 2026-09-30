use chrono::{DateTime, SubsecRound, Utc};

pub fn now() -> String {
    format(Utc::now().trunc_subsecs(3))
}

pub fn epoch() -> String {
    format(DateTime::UNIX_EPOCH)
}

pub fn from_millis(millis: i64) -> Option<String> {
    DateTime::from_timestamp_millis(millis).map(format)
}

pub fn format(datetime: DateTime<Utc>) -> String {
    format!(
        "{}.{:07}Z",
        datetime.format("%Y-%m-%dT%H:%M:%S"),
        datetime.timestamp_subsec_nanos() / 100
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_seven_fractional_digits() {
        assert_eq!(
            from_millis(1_758_800_000_123).as_deref(),
            Some("2025-09-25T11:33:20.1230000Z")
        );
    }

    #[test]
    fn keeps_the_current_time_to_the_millisecond() {
        let now = now();

        assert!(now.ends_with("0000Z"), "{now} should stop at milliseconds");
        assert_eq!(now.len(), "2025-09-25T11:33:20.1230000Z".len());
    }

    #[test]
    fn leaves_out_a_date_it_cannot_represent() {
        assert_eq!(from_millis(i64::MAX), None);
    }
}
