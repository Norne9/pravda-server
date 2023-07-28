pub fn get_days_in_month(year: u16, month: u8) -> u32 {
    use chrono::{Datelike, NaiveDate};
    // Create a NaiveDate representing the first day of the next month
    let next_month = NaiveDate::from_ymd_opt(year as i32, (month as u32) + 1, 1);

    // If the next month exists, subtract 1 day to get the last day of the current month
    if let Some(next_month) = next_month {
        next_month.pred_opt().unwrap().day()
    } else {
        // The next month doesn't exist, so it means we are in December.
        // Return 31 for December.
        31
    }
}

pub fn make_uuid() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

pub fn sha3(text: impl AsRef<str>) -> String {
    use sha3::{Digest, Sha3_256};
    let mut hasher = Sha3_256::new();
    hasher.update(text.as_ref().as_bytes());
    format!("{:x?}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_days_in_month() {
        assert_eq!(get_days_in_month(2023, 1), 31);
        assert_eq!(get_days_in_month(2024, 2), 29);
        assert_eq!(get_days_in_month(2023, 2), 28);
        assert_eq!(get_days_in_month(2023, 3), 31);
        assert_eq!(get_days_in_month(2023, 4), 30);
        assert_eq!(get_days_in_month(2023, 12), 31);
    }

    #[test]
    fn test_make_uuid_uniqueness() {
        assert_ne!(make_uuid(), make_uuid());
    }

    #[test]
    fn test_sha3() {
        assert_ne!(sha3("Qwer4321"), sha3("qwer4321"));
        assert_eq!(sha3("Qwer4321"), sha3("Qwer4321"));
    }
}
