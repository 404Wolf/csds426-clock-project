/// Analyse what fraction of clock offenders cluster near the 32-bit
/// millisecond rollover point (~48.9 days).
///
/// Usage:
///   cargo run --bin rollover-analysis -- [--input PATH] [--raw] [--offender-threshold-s SECS] [--window-days DAYS]
///
/// Use --raw for the full icmp_timestamp.csv (otime/rtime/ttime/rtt_ms columns).
/// Without --raw, expects the enriched format with a clock_offset_ms column.

use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

const MS_PER_DAY: f64 = 86_400_000.0;

/// 2^32 ms in days — the rollover point
const ROLLOVER_DAYS: f64 = (u32::MAX as f64 + 1.0) / MS_PER_DAY; // ≈ 49.710

/// The commonly-observed cluster sits just below the rollover
const TARGET_DAYS: f64 = 48.9;

#[derive(Parser)]
struct Args {
    /// Path to input CSV
    #[arg(long, default_value = "data/icmp_timestamp_enriched_clean.csv")]
    input: PathBuf,

    /// Parse raw icmp_timestamp.csv format (otime/rtime/ttime/rtt_ms) instead of enriched
    #[arg(long)]
    raw: bool,

    /// Minimum absolute offset (seconds) to count as an "offender"
    #[arg(long, default_value_t = 5.0)]
    offender_threshold_s: f64,

    /// Half-width of the window around TARGET_DAYS to call "near rollover" (days)
    #[arg(long, default_value_t = 1.0)]
    window_days: f64,
}

#[derive(Deserialize)]
struct EnrichedRow {
    clock_offset_ms: Option<f64>,
}

#[derive(Deserialize)]
struct RawRow {
    otime: i64,
    rtime: i64,
    ttime: i64,
    rtt_ms: i64,
    ts_nonstandard: u8,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let threshold_ms = args.offender_threshold_s * 1_000.0;
    let window_ms    = args.window_days * MS_PER_DAY;
    let target_ms    = TARGET_DAYS * MS_PER_DAY;

    let mut total         = 0u64;
    let mut with_offset   = 0u64;
    let mut offenders     = 0u64;
    let mut near_rollover = 0u64;

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(&args.input)?;

    if args.raw {
        for result in rdr.deserialize::<RawRow>() {
            let row = match result {
                Ok(r) => r,
                Err(e) => { eprintln!("skipping bad row: {e}"); continue; }
            };
            total += 1;

            if row.ts_nonstandard != 0 { continue; }

            let t4 = row.otime + row.rtt_ms;
            let offset_ms = ((row.rtime - row.otime) + (row.ttime - t4)) / 2;
            with_offset += 1;

            if (offset_ms as f64).abs() <= threshold_ms { continue; }
            offenders += 1;

            if ((offset_ms as f64) - target_ms).abs() <= window_ms {
                near_rollover += 1;
            }
        }
    } else {
        for result in rdr.deserialize::<EnrichedRow>() {
            let row = result?;
            total += 1;

            let Some(offset_ms) = row.clock_offset_ms else { continue };
            with_offset += 1;

            if offset_ms.abs() <= threshold_ms { continue; }
            offenders += 1;

            if (offset_ms - target_ms).abs() <= window_ms {
                near_rollover += 1;
            }
        }
    }

    let pct_offenders_of_measured = offenders as f64 / with_offset as f64 * 100.0;
    let pct_rollover_of_offenders = near_rollover as f64 / offenders as f64 * 100.0;
    let pct_rollover_of_measured  = near_rollover as f64 / with_offset as f64 * 100.0;
    let adjusted_offenders        = offenders - near_rollover;
    let pct_adjusted              = adjusted_offenders as f64 / with_offset as f64 * 100.0;

    println!("32-bit ms rollover point:  {ROLLOVER_DAYS:.3} days  (2^32 ms)");
    println!("Target cluster:            {TARGET_DAYS} days  (±{} days)", args.window_days);
    println!("Offender threshold:        >{} s", args.offender_threshold_s);
    println!();
    println!("Total rows:                {total}");
    println!("Rows with offset:          {with_offset}");
    println!("Offenders (>5s):           {offenders}  ({pct_offenders_of_measured:.2}% of measured)");
    println!("Near {TARGET_DAYS}-day cluster:     {near_rollover}");
    println!("  → {pct_rollover_of_offenders:.2}% of offenders");
    println!("  → {pct_rollover_of_measured:.2}% of all measured hosts");
    println!();
    println!("Offenders excl. cluster:   {adjusted_offenders}  ({pct_adjusted:.2}% of measured)");

    Ok(())
}
