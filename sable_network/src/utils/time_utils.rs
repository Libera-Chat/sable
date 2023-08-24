use chrono::prelude::*;

pub fn now() -> i64 {
    Utc::now().timestamp()
}
