use anyhow::Result;
use chrono::{DateTime, TimeDelta, Timelike, Utc};
use log::{info, trace};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use ureq::Agent;
use ureq::http::Method;
use ureq::tls::{TlsConfig, TlsProvider};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Record {
    /// URL of the host that was probed.
    pub host: String,
    /// Which search round this probe belongs to.
    pub run_num: u32,
    /// 1-indexed position within the round (sorted by offset_micros).
    pub request_num: u32,
    /// Scheduled send offset from the local second boundary, in microseconds.
    pub offset_micros: i64,
    /// Server-reported time from the HTTP Date header (second resolution).
    pub server: DateTime<Utc>,
    /// Local time the request was sent.
    pub send_at: DateTime<Utc>,
    /// Local time the response was received.
    pub receive_at: DateTime<Utc>,
}

pub struct BoundaryPair {
    /// Last probe whose server time was in the earlier second.
    pub before: Record,
    /// First probe whose server time was in the later second.
    pub after: Record,
}

/// Result for a single search round.
pub struct RoundResult {
    pub round_num: u32,
    /// Clock offset estimate from this round's boundary pair (None if no boundary found).
    pub diff_ms: Option<i64>,
    /// Half-span of the search window for this round, in milliseconds.
    pub window_half_ms: i64,
    /// Center of the search window for this round, in microseconds from the local second boundary.
    pub center_us: i64,
}

/// Full result of an HTTP clock measurement.
pub struct MeasurementResult {
    /// Final clock offset (None = frozen clock — no second boundary found in first round).
    pub offset: Option<TimeDelta>,
    /// Per-round details, in round order.
    pub rounds: Vec<RoundResult>,
    /// Every individual probe fired during the measurement, in round then request order.
    pub probes: Vec<Record>,
}

/// Tuning parameters for the binary-search clock measurement.
#[derive(Clone, Debug)]
pub struct SearchConfig {
    /// Number of binary-search rounds to run.
    pub num_rounds: u32,
    /// Number of probes fired per round, spread uniformly across the window.
    pub probes: i64,
    /// Initial half-span of the search window in microseconds.
    pub initial_half_span_us: i64,
    /// Stop recursing when the step between probes drops below this (µs).
    /// Below ~1ms the spacing is smaller than typical HTTP RTT jitter.
    pub min_step_us: i64,
    /// If the server's clock differs from local by more than this many seconds
    /// on the first request, skip binary search and report the raw offset.
    pub sanity_max_offset_secs: i64,
    /// Factor by which to shrink the search window each round (default 2).
    pub shrink_factor: i64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            num_rounds: 10,
            probes: 10,
            initial_half_span_us: 1_300_000,
            min_step_us: 1_000,
            sanity_max_offset_secs: 5,
            shrink_factor: 2,
        }
    }
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
            .timeout_connect(Some(std::time::Duration::from_secs(5)))
            .timeout_send_request(Some(std::time::Duration::from_secs(10)))
            .timeout_recv_response(Some(std::time::Duration::from_secs(10)))
            .http_status_as_error(false)
            .redirect_auth_headers(ureq::config::RedirectAuthHeaders::SameHost)
            .build(),
    )
}

/// Sleep until `fire_at`, then fire a request and return (server_time, sent_at, receive_at).
pub fn sleep_until_and_get_date(
    agent: &Agent,
    url: &str,
    fire_at: DateTime<Utc>,
    method: &str,
) -> Result<(DateTime<Utc>, DateTime<Utc>, DateTime<Utc>)> {
    let sleep_us = (fire_at - chrono::Utc::now())
        .num_microseconds()
        .unwrap_or(0)
        .max(0);
    spin_sleep::sleep(std::time::Duration::from_micros(sleep_us as u64));

    let sent_at = chrono::Utc::now();
    let base = url.split('?').next().unwrap_or(url);
    trace!("{method} {base} send {}", sent_at.format("%H:%M:%S.%6f"));

    let (server, receive_at) = request_http_date(agent, url, method)?;
    Ok((server, sent_at, receive_at))
}

fn call_method(agent: &Agent, url: &str, method: &str) -> Result<ureq::http::Response<ureq::Body>> {
    let m = Method::from_bytes(method.as_bytes())
        .map_err(|e| anyhow::anyhow!("invalid HTTP method: {e}"))?;
    let req = ureq::http::Request::builder().method(m).uri(url).body(())?;
    Ok(agent.run(
        agent
            .configure_request(req)
            .allow_non_standard_methods(true)
            .build(),
    )?)
}

pub fn request_http_date(
    agent: &Agent,
    url: &str,
    method: &str,
) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
    let resp = call_method(agent, url, method)?;
    let receive_at = chrono::Utc::now();

    let date_header = resp
        .headers()
        .get("date")
        .ok_or_else(|| anyhow::anyhow!("no date header"))?
        .to_str()?;

    let server = DateTime::<Utc>::from(httpdate::parse_http_date(date_header)?);
    let base = url.split('?').next().unwrap_or(url);
    trace!(
        "{method} {base} recv {} server {}",
        receive_at.format("%H:%M:%S.%6f"),
        server.format("%H:%M:%S")
    );
    Ok((server, receive_at))
}

pub fn clock_diff_for_pair(pair: &BoundaryPair) -> TimeDelta {
    // The server reports the floor of its time, so when it says :52 it just ticked
    // over to :52.000. Use the "after" probe with the server at exactly that boundary.
    let rtt = ((pair.after.receive_at - pair.after.send_at)
        + (pair.before.receive_at - pair.before.send_at))
        / 2;
    pair.after.server - (pair.after.send_at + rtt / 2)
}

/// Run one full HTTP clock measurement against a URL using HEAD requests and default config.
/// Returns a `MeasurementResult` with per-round details.
/// `offset` is None if the server appears to have a frozen clock.
pub fn measure_host(url: &str) -> Result<MeasurementResult> {
    measure_host_with_config(url, "HEAD", &SearchConfig::default())
}

/// Returns the next local second boundary that is at least `half_span_us` in
/// the future, so that probes at offsets down to `-half_span_us` all have
/// non-negative sleep times.
fn next_boundary(half_span_us: i64) -> DateTime<Utc> {
    let now = chrono::Utc::now();
    let micros_until_next = 1_000_000i64 - now.timestamp_subsec_micros() as i64;
    let extra_seconds = ((half_span_us - micros_until_next).max(0) + 999_999) / 1_000_000;
    now + TimeDelta::microseconds(micros_until_next + extra_seconds * 1_000_000)
}

/// Returns the absolute fire time for each probe in a round, evenly spaced
/// across [boundary + center_us ± half_span_us].
pub fn probe_fire_times(
    boundary: DateTime<Utc>,
    center_us: i64,
    half_span_us: i64,
    num_probes: i64,
) -> Vec<DateTime<Utc>> {
    let step = (2 * half_span_us) / (num_probes - 1);
    (0..num_probes)
        .map(|n| boundary + TimeDelta::microseconds(center_us - half_span_us + n * step))
        .collect()
}

/// Probe `cfg.probes` offsets uniformly across [center ± half_span],
/// then recurse with the span halved and the center narrowed to the boundary found.
/// Returns `(final_pair, per_round_results, all_probes)` in round order.
fn search(
    agent: &Agent,
    url: &str,
    method: &str,
    center_us: i64,
    half_span_us: i64,
    rounds_left: u32,
    round_num: u32,
    cfg: &SearchConfig,
) -> (Option<BoundaryPair>, Vec<RoundResult>, Vec<Record>) {
    let step = (2 * half_span_us) / (cfg.probes - 1);

    if rounds_left == 0 || step < cfg.min_step_us {
        return (None, vec![], vec![]);
    }

    let boundary = next_boundary(half_span_us);
    let fire_times = probe_fire_times(boundary, center_us, half_span_us, cfg.probes);

    let mut rows: Vec<Record> = fire_times
        .into_par_iter()
        .enumerate()
        .filter_map(|(n, fire_at)| {
            let offset = center_us - half_span_us + n as i64 * step;
            let req_url = format!("{}?q={}", url, rand::random::<u64>());
            let (server, send_at, receive_at) =
                sleep_until_and_get_date(agent, &req_url, fire_at, method).ok()?;

            Some(Record {
                host: req_url.clone(),
                run_num: round_num,
                request_num: 0, // assigned after sort below
                offset_micros: offset,
                server,
                send_at,
                receive_at,
            })
        })
        .collect();

    rows.sort_by_key(|r| (r.offset_micros, r.server));
    for (i, row) in rows.iter_mut().enumerate() {
        row.request_num = (i + 1) as u32;
    }

    if let Some(pair) = rows.windows(2).find_map(|w| {
        (w[0].server.second() != w[1].server.second()).then_some(BoundaryPair {
            before: w[0].clone(),
            after: w[1].clone(),
        })
    }) {
        let new_center = (pair.before.offset_micros + pair.after.offset_micros) / 2;
        let diff_ms = clock_diff_for_pair(&pair).num_milliseconds();
        info!(
            "round {}: HIT boundary at {}µs (±{}ms window), diff={}ms",
            round_num,
            new_center,
            half_span_us / 1000,
            diff_ms
        );
        let round_result = RoundResult {
            round_num,
            diff_ms: Some(diff_ms),
            window_half_ms: half_span_us / 1000,
            center_us: new_center,
        };
        let (final_pair, mut rest, mut sub_probes) = search(
            agent,
            url,
            method,
            new_center,
            half_span_us / cfg.shrink_factor,
            rounds_left - 1,
            round_num + 1,
            cfg,
        );
        rest.insert(0, round_result);
        let mut all_probes = rows;
        all_probes.append(&mut sub_probes);
        (Some(final_pair.unwrap_or(pair)), rest, all_probes)
    } else {
        info!(
            "round {}: miss (±{}ms window)",
            round_num,
            half_span_us / 1000
        );
        let round_result = RoundResult {
            round_num,
            diff_ms: None,
            window_half_ms: half_span_us / 1000,
            center_us,
        };
        if round_num == 1 {
            return (None, vec![round_result], rows); // frozen clock — bail immediately
        }
        let (final_pair, mut rest, mut sub_probes) = search(
            agent,
            url,
            method,
            center_us,
            half_span_us / cfg.shrink_factor,
            rounds_left - 1,
            round_num + 1,
            cfg,
        );
        rest.insert(0, round_result);
        let mut all_probes = rows;
        all_probes.append(&mut sub_probes);
        (final_pair, rest, all_probes)
    }
}

/// Run one full HTTP clock measurement against a URL using the given method and config.
/// Uses recursive binary search to home in on the second boundary, then computes
/// the clock offset from the tightest boundary pair found.
pub fn measure_host_with_config(
    url: &str,
    method: &str,
    cfg: &SearchConfig,
) -> Result<MeasurementResult> {
    let agent = make_agent();

    let (sanity_date, _) = request_http_date(&agent, url, method)?;
    let now = chrono::Utc::now();
    if (sanity_date - now).num_seconds().abs() > cfg.sanity_max_offset_secs {
        return Ok(MeasurementResult {
            offset: Some(sanity_date - now),
            rounds: vec![],
            probes: vec![],
        });
    }

    let (pair, rounds, probes) = search(
        &agent,
        url,
        method,
        0,
        cfg.initial_half_span_us,
        cfg.num_rounds,
        1,
        cfg,
    );
    match pair {
        None => {
            info!("{url} appears to have a frozen clock");
            Ok(MeasurementResult {
                offset: None,
                rounds,
                probes,
            })
        }
        Some(pair) => Ok(MeasurementResult {
            offset: Some(clock_diff_for_pair(&pair)),
            rounds,
            probes,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // Fixed boundary: 2026-01-01 00:00:10.000000 UTC
    fn boundary() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 10).unwrap()
    }

    #[test]
    fn half_span_500ms_center_0_3_probes() {
        // half_span=500ms, center=0 → probes at -500ms, 0, +500ms from boundary
        let b = boundary();
        let times = probe_fire_times(b, 0, 500_000, 3);

        assert_eq!(times.len(), 3);
        assert_eq!(times[0], b + TimeDelta::microseconds(-500_000)); // 09.500
        assert_eq!(times[1], b + TimeDelta::microseconds(0)); // 10.000
        assert_eq!(times[2], b + TimeDelta::microseconds(500_000)); // 10.500
    }

    #[test]
    fn half_span_2500ms_center_0_6_probes() {
        // half_span=2.5s, center=0 → first probe at boundary-2.5s, last at boundary+2.5s
        // total span is 5s, step = 5_000_000 / 5 = 1_000_000µs = 1s
        let b = boundary();
        let times = probe_fire_times(b, 0, 2_500_000, 6);

        assert_eq!(times.len(), 6);
        assert_eq!(times[0], b + TimeDelta::microseconds(-2_500_000)); // 07.500
        assert_eq!(times[1], b + TimeDelta::microseconds(-1_500_000)); // 08.500
        assert_eq!(times[2], b + TimeDelta::microseconds(-500_000)); // 09.500
        assert_eq!(times[3], b + TimeDelta::microseconds(500_000)); // 10.500
        assert_eq!(times[4], b + TimeDelta::microseconds(1_500_000)); // 11.500
        assert_eq!(times[5], b + TimeDelta::microseconds(2_500_000)); // 12.500
    }

    #[test]
    fn half_span_with_center_offset() {
        // center=200ms, half_span=100ms → probes at 100ms, 200ms, 300ms from boundary
        let b = boundary();
        let times = probe_fire_times(b, 200_000, 100_000, 3);

        assert_eq!(times.len(), 3);
        assert_eq!(times[0], b + TimeDelta::microseconds(100_000)); // 10.100
        assert_eq!(times[1], b + TimeDelta::microseconds(200_000)); // 10.200
        assert_eq!(times[2], b + TimeDelta::microseconds(300_000)); // 10.300
    }

    #[test]
    fn next_boundary_small_half_span() {
        // half_span=100ms, should find the very next second boundary
        let before = Utc::now();
        let b = next_boundary(100_000);
        let ahead_us = (b - before).num_microseconds().unwrap();

        assert!(
            ahead_us >= 100_000,
            "boundary only {ahead_us}µs ahead, need ≥100000"
        );
        assert!(
            ahead_us < 2_000_000,
            "boundary {ahead_us}µs ahead, should be <2s"
        );
        assert_eq!(
            b.timestamp_subsec_micros(),
            0,
            "boundary not on an exact second"
        );
    }

    #[test]
    fn next_boundary_large_half_span() {
        // half_span=2.5s, must skip ahead enough seconds so the earliest probe
        // (at boundary - 2.5s) is still in the future
        let before = Utc::now();
        let b = next_boundary(2_500_000);
        let ahead_us = (b - before).num_microseconds().unwrap();

        assert!(
            ahead_us >= 2_500_000,
            "boundary only {ahead_us}µs ahead, need ≥2500000"
        );
        assert!(
            ahead_us < 4_500_000,
            "boundary {ahead_us}µs ahead, should be <4.5s"
        );
        assert_eq!(
            b.timestamp_subsec_micros(),
            0,
            "boundary not on an exact second"
        );
    }
}
