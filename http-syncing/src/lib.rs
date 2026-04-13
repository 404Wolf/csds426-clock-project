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
}

const SANITY_CHECK_MAX_OFFSET_SECS: i64 = 5;
const INITIAL_HALF_SPAN_US: i64 = 1_300_000;
const PROBES: i64 = 10;
pub const NUM_ROUNDS: u32 = 10;
/// Stop recursing when the step between probes drops below this threshold.
/// Below ~1ms the probe spacing is smaller than typical HTTP RTT jitter, so
/// any "boundary" found is noise rather than a real clock measurement.
const MIN_STEP_US: i64 = 1_000; // ~1ms; below this, RTT jitter dominates probe spacing

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
    method: &str,
) -> Result<(DateTime<Utc>, DateTime<Utc>, DateTime<Utc>)> {
    let now = chrono::Utc::now();
    let micros_until_next_second = 1_000_000i64 - (now.timestamp_subsec_micros() as i64);
    let sleep_us = (micros_until_next_second + offset_micros).max(0);

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

/// Run one full HTTP clock measurement against a URL using HEAD requests.
/// Returns a `MeasurementResult` with per-round details.
/// `offset` is None if the server appears to have a frozen clock.
pub fn measure_host(url: &str) -> Result<MeasurementResult> {
    measure_host_with_method(url, "HEAD", NUM_ROUNDS)
}

/// Probe `PROBES` offsets uniformly across [center ± half_span],
/// then recurse with the span halved and the center narrowed to the boundary found.
/// Returns `(final_pair, per_round_results)` in round order.
fn search(
    agent: &Agent,
    url: &str,
    method: &str,
    center_us: i64,
    half_span_us: i64,
    rounds_left: u32,
    round_num: u32,
) -> (Option<BoundaryPair>, Vec<RoundResult>) {
    let step = (2 * half_span_us) / (PROBES - 1);

    if rounds_left == 0 || step < MIN_STEP_US {
        return (None, vec![]);
    }

    let mut rows: Vec<Record> = (0..PROBES)
        .into_par_iter()
        .filter_map(|n| {
            let offset = center_us - half_span_us + n * step;
            let req_url = format!("{}?q={}", url, rand::random::<u64>());
            let (server, send_at, receive_at) =
                sleep_to_edge_and_get_date(agent, &req_url, offset, method).ok()?;

            Some(Record {
                host: url.to_string(),
                run_num: round_num,
                offset_micros: offset,
                server,
                send_at,
                receive_at,
            })
        })
        .collect();

    rows.sort_by_key(|r| (r.offset_micros, r.server));

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
            round_num, new_center, half_span_us / 1000, diff_ms
        );
        let round_result = RoundResult {
            round_num,
            diff_ms: Some(diff_ms),
            window_half_ms: half_span_us / 1000,
            center_us: new_center,
        };
        let (final_pair, mut rest) = search(
            agent, url, method, new_center, half_span_us / 2, rounds_left - 1, round_num + 1,
        );
        rest.insert(0, round_result);
        (Some(final_pair.unwrap_or(pair)), rest)
    } else {
        info!("round {}: miss (±{}ms window)", round_num, half_span_us / 1000);
        let round_result = RoundResult {
            round_num,
            diff_ms: None,
            window_half_ms: half_span_us / 1000,
            center_us,
        };
        if round_num == 1 {
            return (None, vec![round_result]); // frozen clock — bail immediately
        }
        let (final_pair, mut rest) = search(
            agent, url, method, center_us, half_span_us / 2, rounds_left - 1, round_num + 1,
        );
        rest.insert(0, round_result);
        (final_pair, rest)
    }
}

/// Run one full HTTP clock measurement against a URL using the given HTTP method.
/// Uses recursive binary search to home in on the second boundary, then computes
/// the clock offset from the tightest boundary pair found.
pub fn measure_host_with_method(url: &str, method: &str, num_rounds: u32) -> Result<MeasurementResult> {
    let agent = make_agent();

    let (sanity_date, _) = request_http_date(&agent, url, method)?;
    let now = chrono::Utc::now();
    if (sanity_date - now).num_seconds().abs() > SANITY_CHECK_MAX_OFFSET_SECS {
        return Ok(MeasurementResult {
            offset: Some(sanity_date - now),
            rounds: vec![],
        });
    }

    let (pair, rounds) = search(&agent, url, method, 0, INITIAL_HALF_SPAN_US, num_rounds, 1);
    match pair {
        None => {
            info!("{url} appears to have a frozen clock");
            Ok(MeasurementResult { offset: None, rounds })
        }
        Some(pair) => Ok(MeasurementResult {
            offset: Some(clock_diff_for_pair(&pair)),
            rounds,
        }),
    }
}
