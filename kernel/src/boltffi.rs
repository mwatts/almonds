use boltffi::*;
use chrono::{DateTime, FixedOffset};

custom_type!(
  pub  DateTimeWithTimeZone,
    remote = DateTime<FixedOffset>,
    repr = i64,
    into_ffi = |dt: &DateTime<FixedOffset>| dt.timestamp_millis(),
    try_from_ffi = |millis: i64| {
        DateTime::from_timestamp_millis(millis)
            .map(|dt_utc| dt_utc.fixed_offset())
            .ok_or(CustomTypeConversionError)
    },
);
