use chrono::{TimeDelta, Timelike};
use clocks::Record;
use itertools::Itertools;
use rayon::prelude::*;

fn clock_diff_for_pair(right_before: &Record, right_after: &Record) -> TimeDelta {
    let rtt_before = right_before.receive_at - right_before.send_at;
    let rtt_after = right_after.receive_at - right_after.send_at;

    // For each request: server clock read N at local time (send_at + rtt/2)
    let diff_before = right_before.server - (right_before.send_at + rtt_before / 2);
    let diff_after = right_after.server - (right_after.send_at + rtt_after / 2);

    (diff_before + diff_after) / 2
}

fn avg_clock_diff_for_host(_host: &str, rows: &[&Record]) -> Option<TimeDelta> {
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

fn main() {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(std::io::stdin());

    let rows: Vec<Record> = rdr
        .deserialize()
        .map(|r| r.expect("failed to parse row"))
        .collect();

    let groups = rows.iter().into_group_map_by(|r| r.host.clone());

    let mut results: Vec<(String, TimeDelta)> = groups
        .par_iter()
        .filter_map(|(host, group)| avg_clock_diff_for_host(host, group).map(|d| (host.clone(), d)))
        .collect();

    results.sort_by(|a, b| a.0.cmp(&b.0));

    for (host, diff) in results {
        println!("{host}: min_clock_diff_ms={}", diff.num_milliseconds());
    }
}
