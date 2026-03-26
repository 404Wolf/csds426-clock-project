use chrono::{DateTime, TimeDelta, Timelike, Utc};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use ureq::Agent;
use ureq::tls::{TlsConfig, TlsProvider};

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    pub host: String,
    pub run_num: u32,
    pub offset_micros: i64,
    pub server: DateTime<Utc>,
    pub send_at: DateTime<Utc>,
    pub receive_at: DateTime<Utc>,
}

pub fn make_agent() -> Agent {
    Agent::new_with_config(
        ureq::config::Config::builder()
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::Rustls)
                    .disable_verification(true)
                    .build(),
            )
            .http_status_as_error(false)
            .redirect_auth_headers(ureq::config::RedirectAuthHeaders::SameHost)
            .build(),
    )
}

pub fn sleep_to_edge_and_get_date(
    agent: &Agent,
    url: &str,
    offset_micros: i64,
) -> Result<(DateTime<Utc>, DateTime<Utc>, DateTime<Utc>), Box<dyn std::error::Error + Send + Sync>>
{
    let time_now = chrono::Utc::now();
    let micros_until_next_second =
        1_000_000i64 - (time_now.timestamp_subsec_micros() as i64 % 1_000_000i64);
    let total_micros = (micros_until_next_second + offset_micros).max(0);

    spin_sleep::sleep(std::time::Duration::from_micros(total_micros as u64));
    let sent_at = chrono::Utc::now();

    let resp = agent.head(url).call()?;
    let receive_at = chrono::Utc::now();

    let date_header = resp
        .headers()
        .get("date")
        .ok_or("no date header")?
        .to_str()?;

    let reported_date = httpdate::parse_http_date(date_header)?;
    let reported_date: DateTime<Utc> = DateTime::<Utc>::from(reported_date);

    Ok((reported_date, sent_at, receive_at))
}

pub fn estimate_rtt(
    agent: &Agent,
    url: &str,
) -> Result<u128, Box<dyn std::error::Error + Send + Sync>> {
    let mut rtt_sum_micros: u128 = 0;

    for _ in 0..5 {
        let start = std::time::Instant::now();
        agent.head(url).call()?;
        rtt_sum_micros += start.elapsed().as_micros();
    }

    Ok(rtt_sum_micros / 5)
}

pub fn clock_diff_for_pair(right_before: &Record, right_after: &Record) -> TimeDelta {
    let rtt_before = right_before.receive_at - right_before.send_at;
    let rtt_after = right_after.receive_at - right_after.send_at;
    let diff_before = right_before.server - (right_before.send_at + rtt_before / 2);
    let diff_after = right_after.server - (right_after.send_at + rtt_after / 2);
    (diff_before + diff_after) / 2
}

pub fn avg_clock_diff_for_host(_host: &str, rows: &[&Record]) -> Option<TimeDelta> {
    rows.iter()
        .into_group_map_by(|r| r.run_num)
        .into_iter()
        .filter_map(|(_run_num, group)| {
            let mut run_rows: Vec<&Record> = group.into_iter().copied().collect();
            run_rows.sort_by_key(|r| r.receive_at);

            let (right_before, right_after) = run_rows.windows(2).find_map(|w| {
                (w[0].receive_at.second() > w[1].receive_at.second()).then_some((w[0], w[1]))
            })?;

            Some(clock_diff_for_pair(right_before, right_after))
        })
        .min()
}

/// Run one full HTTP clock measurement against a URL: estimate RTT, fire 100 probes
/// around the second boundary, and compute clock offset.
/// Returns `(rtt_us, clock_offset)` or `None` on failure.
pub fn measure_host(url: &str) -> Option<(u128, TimeDelta)> {
    let agent = make_agent();
    let rtt_estimate = estimate_rtt(&agent, url).ok()?;

    let step_micros: i64 = 300;
    let entries: i64 = 100;
    let rtt_i64 = rtt_estimate as i64;
    let center = -rtt_i64;
    let half_span = (entries / 2) * step_micros;
    let jitter = rand::random::<i64>().abs() % step_micros.max(1);
    let start = (center - half_span) + jitter;

    let hit_at: Vec<i64> = (0..entries).map(|n| start + n * step_micros).collect();

    let mut rows: Vec<Record> = hit_at
        .par_iter()
        .filter_map(|&i| {
            let req_url = format!("{}?q={}", url, rand::random::<u64>());
            let (server, sent_at, receive_at) =
                sleep_to_edge_and_get_date(&agent, &req_url, i).ok()?;
            Some(Record {
                host: url.to_string(),
                run_num: 0,
                offset_micros: i,
                server,
                send_at: sent_at,
                receive_at,
            })
        })
        .collect();

    rows.sort_by_key(|r| r.receive_at);

    let row_refs: Vec<&Record> = rows.iter().collect();
    let diff = avg_clock_diff_for_host(url, &row_refs)?;

    Some((rtt_estimate, diff))
}
