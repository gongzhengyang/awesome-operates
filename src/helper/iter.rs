use serde_json::map::Iter;
use serde_json::Value;
use snafu::OptionExt;

use crate::error::{OptionNoneSnafu, Result};

/// iter an object
/// ```rust
///  use awesome_operates::helper::iter_object;
///  let obj = serde_json::json!({"a": {"b": 1}});
///  if let Ok(iter) = iter_object(&obj, "a") {
///     for (key, value) in iter {
///         println!("{key}: {value}")
///     }
///  }
/// ```
pub fn iter_object<'a>(value: &'a Value, key: &'a str) -> Result<Iter<'a>> {
    Ok(value.as_object().context(OptionNoneSnafu)?[key]
        .as_object()
        .context(OptionNoneSnafu)?
        .iter())
}
