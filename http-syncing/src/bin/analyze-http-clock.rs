use chrono::TimeDelta;
use clocks::Record;
use itertools::Itertools;
use rayon::prelude::*;

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
        .filter_map(|(host, group)| {
            clocks::avg_clock_diff_for_host(host, group).map(|d| (host.clone(), d))
        })
        .collect();

    results.sort_by(|a, b| a.0.cmp(&b.0));

    for (host, diff) in results {
        println!("{host}: min_clock_diff_ms={}", diff.num_milliseconds());
    }
}
