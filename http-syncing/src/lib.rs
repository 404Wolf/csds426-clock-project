use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    pub host: String,
    pub offset_micros: i64,
    pub server: DateTime<Utc>,
    pub send_at: DateTime<Utc>,
    pub receive_at: DateTime<Utc>,
}
