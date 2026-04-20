use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use serde_json::json;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trip {
    depart: NaiveDate,
    ret: NaiveDate,
    description: String,
}

impl Trip {
    pub fn new(depart: NaiveDate, ret: NaiveDate, description: String) -> Self {
        Trip {
            depart,
            ret,
            description,
        }
    }

    // Difference in days between depart and return, exclusive.
    pub fn interval(&self) -> usize {
        self.ret.signed_duration_since(self.depart).num_days() as usize - 1
    }

    pub fn overlap(&self, trip: &Trip) -> bool {
        trip.depart < self.ret && self.depart < trip.ret
    }

    pub fn get_depart(&self) -> NaiveDate {
        self.depart
    }

    pub fn get_return(&self) -> NaiveDate {
        self.ret
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn to_json(&self) -> String {
        json!({
            "depart": self.depart.to_string(),
            "ret": self.ret.to_string(),
            "description": self.description,
        }).to_string()
    }

    pub fn from_json(json: &str) -> Result<Trip, String> {
        let trip: Trip = serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;
        Ok(trip)
    }

    pub fn calculate_outside_days(trips: &[Trip]) -> BTreeMap<NaiveDate, usize> {
        let mut all_days_outside = BTreeMap::new();
        for trip in trips {
            let mut d = trip.get_depart() + Duration::days(1);
            while d < trip.get_return() {
                // Get the day one year ago from the current day
                let one_year_ago = d - Duration::days(365);
                // Iterate over the days in the hashmap and count.
                let count = all_days_outside.range(one_year_ago..=d).count();
                
                all_days_outside.insert(d, count + 1);
                d += Duration::days(1);
            }
        }
        all_days_outside
    }

    pub fn export_csv(trips: &[Trip]) -> String {
        let mut csv = String::new();
        csv.push_str("Depart,Return,Description\n");
        for trip in trips {
            csv.push_str(&format!("{},{},\"{}\"\n", trip.get_depart(), trip.get_return(), trip.get_description()));
        }
        csv
    }

    pub fn import_csv(csv: &str) -> Result<Vec<Trip>, String> {
        let mut trips = Vec::new();
        for line in csv.lines() {
            let parts = line.split(',').collect::<Vec<&str>>();
            if parts.len() != 3 {
                continue;
            }

            // Skip header line
            if parts[0] == "Depart" {
                continue;
            }

            let depart = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d").map_err(|e| format!("Invalid date: {}: {}", parts[0], e))?;
            let ret = NaiveDate::parse_from_str(parts[1], "%Y-%m-%d").map_err(|e| format!("Invalid date: {}: {}", parts[1], e))?;
            let description = String::from(parts[2].trim_matches('"'));
            trips.push(Trip::new(depart, ret, description));
        }
        Ok(trips)
    }

    pub fn to_json_array(trips: &[Trip]) -> String {
        serde_json::json!(trips).to_string()
    }

    pub fn from_json_array(json: &str) -> Result<Vec<Trip>, String> {
        let trips = json.split(',').map(|t| Trip::from_json(t)).collect::<Result<Vec<Trip>, String>>()?;
        Ok(trips)
    }
}


#[cfg(test)]
mod test {
    use super::Trip;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn new_creates_trip_with_given_fields() {
        let d = date(2026, 2, 6);
        let r = date(2026, 2, 10);
        let t = Trip::new(d, r, "Holiday".to_string());
        assert_eq!(t.get_depart(), d);
        assert_eq!(t.get_return(), r);
    }

    #[test]
    fn interval_is_full_days_between_exclusive() {
        // Mon 2 Feb -> Wed 4 Feb => only Tue 3 Feb = 1 day
        let t = Trip::new(date(2026, 2, 2), date(2026, 2, 4), String::new());
        assert_eq!(t.interval(), 1);
    }

    #[test]
    fn interval_zero_when_depart_one_day_before_return() {
        let t = Trip::new(date(2026, 2, 6), date(2026, 2, 7), String::new());
        assert_eq!(t.interval(), 0);
    }

    #[test]
    fn no_overlap_when_other_ends_when_we_start() {
        let a = Trip::new(date(2026, 1, 1), date(2026, 1, 5), String::new());
        let b = Trip::new(date(2026, 1, 5), date(2026, 1, 10), String::new());
        assert!(!a.overlap(&b));
        assert!(!b.overlap(&a));
    }

    #[test]
    fn no_overlap_when_other_starts_when_we_end() {
        let a = Trip::new(date(2026, 1, 5), date(2026, 1, 10), String::new());
        let b = Trip::new(date(2026, 1, 1), date(2026, 1, 5), String::new());
        assert!(!a.overlap(&b));
    }

    #[test]
    fn overlap_when_intervals_share_days() {
        let a = Trip::new(date(2026, 1, 1), date(2026, 1, 10), String::new());
        let b = Trip::new(date(2026, 1, 4), date(2026, 1, 6), String::new());
        assert!(a.overlap(&b));
        assert!(b.overlap(&a));
    }

    #[test]
    fn overlap_when_one_fully_contains_other() {
        let a = Trip::new(date(2026, 1, 1), date(2026, 1, 5), String::new());
        let b = Trip::new(date(2026, 1, 2), date(2026, 1, 4), String::new());
        assert!(a.overlap(&b));
        assert!(b.overlap(&a));
    }

    #[test]
    fn calculate_outside_days_returns_correct_number_of_days() {
        let trips = vec![
            Trip::new(date(2026, 1, 1), date(2026, 1, 5), String::new()),
            Trip::new(date(2026, 1, 6), date(2026, 1, 10), String::new()),
        ];
        let outside_days = Trip::calculate_outside_days(&trips);
        assert_eq!(outside_days.len(), 6);

        let expected: Vec<(NaiveDate, Option<usize>)> = vec![
            (date(2026, 1, 1), None),
            (date(2026, 1, 2), Some(1)),
            (date(2026, 1, 3), Some(2)),
            (date(2026, 1, 4), Some(3)),
            (date(2026, 1, 5), None),
            (date(2026, 1, 6), None),
            (date(2026, 1, 7), Some(4)),
            (date(2026, 1, 8), Some(5)),
            (date(2026, 1, 9), Some(6)),
            (date(2026, 1, 10), None),
        ];
        for (d, count) in expected {
            assert_eq!(outside_days.get(&d), count.as_ref(), "date {}", d);
        }
    }

    #[test]
    fn export_csv_import_csv_returns_correct_csv() {
        let trips = vec![
            Trip::new(date(2026, 1, 1), date(2026, 1, 5), "Holiday".to_string()),
            Trip::new(date(2026, 1, 6), date(2026, 1, 10), "Business".to_string()),
        ];
        let csv = Trip::export_csv(&trips);
        assert_eq!(csv, "Depart,Return,Description\n2026-01-01,2026-01-05,\"Holiday\"\n2026-01-06,2026-01-10,\"Business\"\n");

        let imported_trips = Trip::import_csv(&csv).unwrap();
        assert_eq!(imported_trips, trips);
    }
}
