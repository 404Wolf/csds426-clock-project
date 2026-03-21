use clocks::Record;
use itertools::Itertools;

fn analyze_host(host: &str, rows: &[&Record]) {
    let transition = rows
        .windows(2)
        .find(|w| w[0].receive_at.timestamp() != w[1].receive_at.timestamp())
        .unwrap();

    let rtt = transition[1].receive_at - transition[0].send_at;

    println!("{}: {:?}", host, rtt);
}

fn main() {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(std::io::stdin());

    let rows: Vec<Record> = rdr
        .deserialize()
        .map(|r| r.expect("failed to parse row"))
        .collect();

    for (host, group) in &rows.iter().chunk_by(|r| r.host.clone()) {
        let group: Vec<&Record> = group.collect();
        analyze_host(&host, &group);
    }
}
