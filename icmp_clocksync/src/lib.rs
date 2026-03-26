use std::io::BufReader;

use serde::{Deserialize, Serialize};

pub mod shared;

#[derive(Debug, Deserialize, Serialize)]
pub struct IcmpTimestampRecord {
    pub saddr: String,
    pub saddr_raw: u32,
    pub daddr: String,
    pub daddr_raw: u32,
    pub ipid: u16,
    pub ttl: u8,
    #[serde(rename = "type")]
    pub icmp_type: u8,
    pub code: u8,
    pub icmp_id: u16,
    pub seq: u16,
    pub otime: u64,
    pub rtime: u64,
    pub ttime: u64,
    pub rtt_ms: u64,
    pub remote_processing_ms: u64,
    pub classification: String,
    pub success: u8,
    pub repeat: u8,
    pub cooldown: u8,
    pub timestamp_str: String,
    pub timestamp_ts: u64,
    pub timestamp_us: u64,
}

#[derive(Debug, Serialize)]
pub struct EnrichedRecord {
    pub batch_num: u64,
    pub ip: String,
    pub hostname: String,
    pub rtt_ms: f64,
    pub is_http: bool,
    pub had_date: bool,
    pub country: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(flatten)]
    pub source: SourceData,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum SourceData {
    IcmpTimestamp {
        daddr: String,
        otime: u64,
        rtime: u64,
        ttime: u64,
        clock_offset_ms: i64,
    },
    PlainIp {},
}

#[derive(Debug, Deserialize)]
struct BatchOnly {
    batch_num: u64,
}

pub fn get_latest_batch(path: &std::path::Path) -> Option<u64> {
    let file = std::fs::File::open(path).ok()?;
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(std::io::BufReader::new(file));
    let mut max_batch = None;
    for rec in rdr.deserialize::<BatchOnly>().flatten() {
        max_batch = Some(max_batch.map_or(rec.batch_num, |m: u64| m.max(rec.batch_num)));
    }
    max_batch
}

/// Create a streaming CSV deserializer from any reader.
pub fn iter_icmp_csv<R: std::io::Read>(
    reader: R,
) -> csv::DeserializeRecordsIntoIter<BufReader<R>, IcmpTimestampRecord> {
    csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(BufReader::new(reader))
        .into_deserialize()
}
