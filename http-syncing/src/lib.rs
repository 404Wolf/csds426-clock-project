use anyhow::Result;
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

const SANITY_CHECK_MAX_OFFSET_SECS: i64 = 5;

pub fn make_agent() -> Agent {
    Agent::new_with_config(
        ureq::config::Config::builder()
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::Rustls)
                    .disable_verification(true)
                    .build(),
            )
            .timeout_connect(Some(std::time::Duration::from_secs(5)))
            .timeout_send_request(Some(std::time::Duration::from_secs(10)))
            .timeout_recv_response(Some(std::time::Duration::from_secs(10)))
            .http_status_as_error(false)
            .redirect_auth_headers(ureq::config::RedirectAuthHeaders::SameHost)
            .build(),
    )
}

pub fn sleep_to_edge_and_get_date(
    agent: &Agent,
    url: &str,
    offset_micros: i64,
) -> Result<(DateTime<Utc>, DateTime<Utc>, DateTime<Utc>)> {
    let time_now = chrono::Utc::now();
    let micros_until_next_second =
        1_000_000i64 - (time_now.timestamp_subsec_micros() as i64 % 1_000_000i64);
    let total_micros = (micros_until_next_second + offset_micros).max(0);

    spin_sleep::sleep(std::time::Duration::from_micros(total_micros as u64));
    let sent_at = chrono::Utc::now();

    let (reported_date, receive_at, _rtt) = request_http_date(agent, url)?;

    Ok((reported_date, sent_at, receive_at))
}

pub fn request_http_date(agent: &Agent, url: &str) -> Result<(DateTime<Utc>, DateTime<Utc>, u128)> {
    let start = std::time::Instant::now();
    let resp = agent.head(url).call()?;
    let rtt_micros = start.elapsed().as_micros();
    let receive_at = chrono::Utc::now();

    let date_header = resp
        .headers()
        .get("date")
        .ok_or_else(|| anyhow::anyhow!("no date header"))?
        .to_str()?;

    let reported_date = httpdate::parse_http_date(date_header)?;
    let reported_date: DateTime<Utc> = DateTime::<Utc>::from(reported_date);

    Ok((reported_date, receive_at, rtt_micros))
}

pub fn estimate_rtt(agent: &Agent, url: &str) -> Result<u128> {
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
                (w[0].server.second() != w[1].server.second()).then_some((w[0], w[1]))
            })?;

            Some(clock_diff_for_pair(right_before, right_after))
        })
        .min_by_key(|d| d.num_milliseconds().abs())
}

/// Run one full HTTP clock measurement against a URL.
/// Fires multiple bursts (each targeting a successive second boundary) with tight
/// probe spacing, then picks the best boundary crossing.
/// Returns `(rtt_us, clock_offset)` on success.
pub fn measure_host(url: &str) -> Result<(u128, TimeDelta)> {
    let agent = make_agent();

    // Before we proceed, do a simple sanity check: maybe the host is off by
    // more than SANITY_CHECK_MAX_OFFSET_SECS seconds. If so, bail — the
    // measurement would be meaningless.
    let (sanity_date, _, sanity_rtt) = request_http_date(&agent, url)?;
    let now = chrono::Utc::now();
    if (sanity_date - now).num_seconds().abs() > SANITY_CHECK_MAX_OFFSET_SECS {
        // HTTP Date only has second resolution, so this offset is coarse.
        return Ok((sanity_rtt, sanity_date - now));
    }

    let rtt_estimate = estimate_rtt(&agent, url)?;

    // 10 independent chances to catch a second boundary (~1s apart)
    let num_bursts: u32 = 10;
    // 60 parallel HEAD requests per burst, densely covering the window
    let probes_per_burst: i64 = 60;
    // 200µs between each probe's scheduled send time (this is the tightest boundary pair we can find)
    let step_micros: i64 = 200;
    let rtt_i64 = rtt_estimate as i64;
    // probes spread +/- 6ms around center (60/2 * 200us = 6ms); total 12ms
    // window is wide enough to absorb typical RTT jitter
    let half_span = (probes_per_burst / 2) * step_micros;

    let mut all_rows: Vec<Record> = Vec::new();

    for burst in 0..num_bursts {
        // send one RTT before the local second tick so the request
        // arrives at the server right around its second boundary
        let center = -rtt_i64;
        let jitter = rand::random::<i64>().abs() % step_micros.max(1);
        let start = (center - half_span) + jitter;

        let hit_at: Vec<i64> = (0..probes_per_burst)
            .map(|n| start + n * step_micros)
            .collect();

        let mut rows: Vec<Record> = hit_at
            .par_iter()
            .filter_map(|&i| {
                let req_url = format!("{}?q={}", url, rand::random::<u64>());
                let (server, sent_at, receive_at) =
                    sleep_to_edge_and_get_date(&agent, &req_url, i).ok()?;
                Some(Record {
                    host: url.to_string(),
                    run_num: burst,
                    offset_micros: i,
                    server,
                    send_at: sent_at,
                    receive_at,
                })
            })
            .collect();

        let hit = rows
            .iter()
            .map(|r| r.server.second())
            .collect::<std::collections::HashSet<_>>()
            .len()
            > 1;
        eprintln!(
            "  burst {}/{}: {}/{} probes, boundary {}",
            burst + 1,
            num_bursts,
            rows.len(),
            probes_per_burst,
            if hit { "HIT" } else { "miss" }
        );

        rows.sort_by_key(|r| r.receive_at);
        all_rows.append(&mut rows);
    }

    let row_refs: Vec<&Record> = all_rows.iter().collect();
    let diff = avg_clock_diff_for_host(url, &row_refs)
        .ok_or_else(|| anyhow::anyhow!("no second boundary found across {} bursts", num_bursts))?;

    Ok((rtt_estimate, diff))
}
