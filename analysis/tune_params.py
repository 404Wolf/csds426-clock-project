#!/usr/bin/env python3
"""Use optuna to find test-http parameters that minimize clock offset error."""

import argparse
import csv
import os
import re
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass

import optuna

OFFSET_RE = re.compile(r"http_clock_offset_us=(-?\d+)us")

OFFSETS_S = [-1, -0.1, -0.01, 0, 0.01, 0.1, 1]


@dataclass
class SearchParams:
    rounds: int
    probes: int
    half_span_us: int
    min_step_us: int
    shrink_factor: int

    @staticmethod
    def from_trial(trial: optuna.Trial) -> "SearchParams":
        return SearchParams(
            rounds=trial.suggest_int("rounds", 3, 20),
            probes=trial.suggest_int("probes", 3, 30),
            half_span_us=trial.suggest_int("half_span_us", 1_000_000, 5_000_000, step=150_000),
            min_step_us=trial.suggest_int("min_step_us", 0, 5_000, step=100),
            shrink_factor=trial.suggest_int("shrink_factor", 2, 5),
        )

    def to_flags(self) -> list[str]:
        return [
            "--rounds", str(self.rounds),
            "--probes", str(self.probes),
            "--initial-half-span-us", str(self.half_span_us),
            "--min-step-us", str(self.min_step_us),
            "--method", "HEAD",
            "--shrink-factor", str(self.shrink_factor),
        ]


def measure(host: str, offset_s: float, params: SearchParams, timeout: int = 60) -> int | None:
    url = f"http://{host}/{offset_s}"
    cmd = ["just", "test-http", url, "--"] + params.to_flags()
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        m = OFFSET_RE.search(r.stdout + r.stderr)
        return int(m.group(1)) if m else None
    except subprocess.TimeoutExpired:
        return None


MEASUREMENTS_CSV = "tune_measurements.csv"
CSV_FIELDS = ["trial", "host", "offset_s", "expected_us", "measured_us", "err_us",
              "rounds", "probes", "half_span_us", "min_step_us", "method", "shrink_factor"]


def ensure_csv():
    if not os.path.exists(MEASUREMENTS_CSV):
        with open(MEASUREMENTS_CSV, "w", newline="") as f:
            csv.writer(f).writerow(CSV_FIELDS)


def log_measurement(trial: int, host: str, offset_s: float, expected_us: int,
                    measured_us: int, err_us: int, params: SearchParams):
    with open(MEASUREMENTS_CSV, "a", newline="") as f:
        csv.writer(f).writerow([
            trial, host, offset_s, expected_us, measured_us, err_us,
            params.rounds, params.probes, params.half_span_us,
            params.min_step_us, params.shrink_factor,
        ])


def evaluate(hosts: list[str], params: SearchParams, offsets: list[float],
             reps: int = 2, trial_num: int = 0) -> float:
    jobs = [(host, off) for _ in range(reps) for off in offsets for host in hosts]

    def run(job):
        host, off = job
        return host, off, measure(host, off, params)

    with ThreadPoolExecutor(max_workers=len(hosts)) as pool:
        futures = {pool.submit(run, job): job for job in jobs}
        results = []
        for fut in as_completed(futures):
            host, offset_s, result_us = fut.result()
            expected_us = int(offset_s * 1_000_000)
            if result_us is None:
                print(f"    {host} offset={offset_s:+.1f}s -> FAIL")
            else:
                err = abs(result_us - expected_us)
                print(f"    {host} offset={offset_s:+.1f}s -> measured={result_us:+d}us expected={expected_us:+d}us err={err}us")
            results.append((host, offset_s, result_us))

    total_err = 0
    for host, offset_s, result_us in results:
        if result_us is None:
            return 10_000_000
        expected_us = int(offset_s * 1_000_000)
        err = abs(result_us - expected_us)
        total_err += err
        log_measurement(trial_num, host, offset_s, expected_us, result_us, err, params)

    avg_err = total_err / len(results)
    print(f"  => avg_err={avg_err:.0f}us")
    return avg_err


def main():
    ap = argparse.ArgumentParser(description="Tune test-http parameters with optuna")
    ap.add_argument("hosts", nargs="+", help="host:port of fake time servers")
    ap.add_argument("--trials", type=int, default=200000, help="Number of trials")
    ap.add_argument("--reps", type=int, default=2, help="Repetitions per (host, offset) pair")
    ap.add_argument("--offsets", type=float, nargs="+", default=OFFSETS_S,
                     help="Clock offsets to test (seconds)")
    args = ap.parse_args()

    ensure_csv()

    def objective(trial: optuna.Trial) -> float:
        params = SearchParams.from_trial(trial)
        print(f"\nTrial {trial.number}: {params}")
        return evaluate(args.hosts, params, args.offsets, args.reps, trial.number)

    study = optuna.create_study(
        direction="minimize",
        storage="sqlite:///tune_params.db",
        study_name="http-clock-tuner",
        load_if_exists=True,
    )
    try:
        study.optimize(objective, n_trials=args.trials)
    except KeyboardInterrupt:
        print("\nInterrupted, showing results so far...")

    print(f"\n{'='*60}")
    if len(study.trials) > 0 and study.best_trial.value is not None:
        print(f"Best avg error: {study.best_value:.0f}us")
        print(f"Best params: {study.best_params}")
    else:
        print("No completed trials.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
