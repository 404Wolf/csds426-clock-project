#!/usr/bin/env python3
"""Use optuna to find test-http parameters that minimize clock offset error."""

import argparse
import random
import re
import subprocess
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass

import optuna

OFFSET_RE = re.compile(r"http_clock_offset_us=(-?\d+)us")

OFFSETS_S = [-5, -1, -0.5, -0.1, 0.1, 0.5, 1, 5]


@dataclass
class SearchParams:
    rounds: int
    probes: int
    half_span_us: int
    min_step_us: int
    method: str
    shrink_factor: int

    @staticmethod
    def from_trial(trial: optuna.Trial) -> "SearchParams":
        return SearchParams(
            rounds=trial.suggest_int("rounds", 3, 30),
            probes=trial.suggest_int("probes", 3, 20),
            half_span_us=trial.suggest_int("half_span_us", 2_000_000, 5_000_000, step=100_000),
            min_step_us=trial.suggest_int("min_step_us", 0, 5_000, step=100),
            method=trial.suggest_categorical("method", ["HEAD", "GET"]),
            shrink_factor=trial.suggest_int("shrink_factor", 2, 5),
        )

    def to_flags(self) -> list[str]:
        return [
            "--rounds", str(self.rounds),
            "--probes", str(self.probes),
            "--initial-half-span-us", str(self.half_span_us),
            "--min-step-us", str(self.min_step_us),
            "--method", self.method,
            "--shrink-factor", str(self.shrink_factor),
        ]


def measure(host: str, offset_s: float, params: SearchParams, timeout: int = 60) -> int | None:
    """Run test-http with offset baked into the URL, return measured offset in microseconds."""
    url = f"http://{host}/{offset_s}"
    cmd = ["just", "test-http", url, "--"] + params.to_flags()
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        m = OFFSET_RE.search(r.stdout + r.stderr)
        return int(m.group(1)) if m else None
    except subprocess.TimeoutExpired:
        return None


def run_one(host: str, offset_s: float, params: SearchParams) -> tuple[str, float, int | None]:
    result = measure(host, offset_s, params)
    return host, offset_s, result


def evaluate(hosts: list[str], params: SearchParams, offsets: list[float]) -> float:
    """Test params across all offsets and hosts fully in parallel."""
    jobs = [(host, off) for off in offsets for host in hosts]

    with ThreadPoolExecutor(max_workers=len(jobs)) as pool:
        futures = [pool.submit(run_one, host, off, params) for host, off in jobs]
        results = [f.result() for f in futures]

    total_err = 0
    for host, offset_s, result_us in sorted(results, key=lambda r: (r[1], r[0])):
        expected_us = int(offset_s * 1_000_000)
        if result_us is None:
            print(f"    {host} offset={offset_s:+.1f}s -> FAIL")
            return 10_000_000
        err = abs(result_us - expected_us)
        total_err += err
        print(f"    {host} offset={offset_s:+.1f}s -> measured={result_us:+d}us expected={expected_us:+d}us err={err}us")

    avg_err = total_err / len(results)
    print(f"  => avg_err={avg_err:.0f}us")
    return avg_err


def main():
    ap = argparse.ArgumentParser(description="Tune test-http parameters with optuna")
    ap.add_argument("hosts", nargs="+", help="host:port of fake time servers")
    ap.add_argument("--trials", type=int, default=2000, help="Number of trials")
    ap.add_argument("--jobs", type=int, default=1, help="Parallel trials")
    ap.add_argument("--offsets", type=float, nargs="+", default=OFFSETS_S,
                     help="Clock offsets to test (seconds)")
    args = ap.parse_args()

    def objective(trial: optuna.Trial) -> float:
        params = SearchParams.from_trial(trial)
        return evaluate(args.hosts, params, args.offsets)

    study = optuna.create_study(
        direction="minimize",
        storage="sqlite:///tune_params.db",
        study_name="http-clock-tuner",
        load_if_exists=True,
    )
    try:
        study.optimize(objective, n_trials=args.trials, n_jobs=args.jobs)
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
