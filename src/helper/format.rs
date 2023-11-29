use chrono::TimeZone;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use crate::consts;

pub fn default_formatted_now() -> String {
    formatted_now(consts::DEFAULT_TIME_FORMAT)
}

pub fn formatted_now(fmt: &str) -> String {
    chrono::Local::now().format(fmt).to_string()
}

pub fn format_from_timestamp(value: u64) -> String {
    chrono::Utc
        .timestamp_opt(value as i64, 0)
        .unwrap()
        .format(consts::DEFAULT_TIME_FORMAT)
        .to_string()
}

#[inline]
pub fn human_bytes(value: u64) -> String {
    let mut value = value as f64;
    if value < 1024.0 {
        return format!("{value:.2} B");
    }
    for symbol in ["KB", "MB", "GB", "TB"] {
        value /= 1024.0;
        if value < 1024.0 {
            return format!("{value:.2} {symbol}");
        }
    }
    format!("{value}")
}

#[inline]
pub fn decimal_with_two(value: f64) -> Decimal {
    Decimal::from_f64(value).unwrap_or_default().round_dp(2)
}

#[inline]
pub fn decimal_with_four(value: f64) -> Decimal {
    Decimal::from_f64(value).unwrap_or_default().round_dp(4)
}
