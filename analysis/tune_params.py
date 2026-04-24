#!/usr/bin/env python3
"""Use optuna to find test-http parameters that minimize clock offset error across multiple clock offsets."""

import argparse
import re
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass

import optuna

OFFSET_RE = re.compile(r"http_clock_offset_us=(-?\d+)us")

OFFSETS_S = [-5, -1, -0.5, -0.1, 0.1, 0.5, 1, 5]


@dataclass
class Server:
    host: str
    ssh: str  # e.g. "root@1.2.3.4"

    def ssh_cmd(self, cmd: str) -> None:
        subprocess.run(["ssh", self.ssh, cmd], capture_output=True, timeout=15)

    def nudge_time(self, offset_s: float) -> None:
        self.ssh_cmd("chronyc makestep")
        time.sleep(1)
        if offset_s != 0:
            # Use absolute time to avoid compounding
            target = time.time() + offset_s
            self.ssh_cmd(f"date -s @{target:.3f}")


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


def measure(host: str, params: SearchParams, timeout: int = 60) -> int | None:
    """Run test-http and return measured offset in microseconds, or None on failure."""
    cmd = ["just", "test-http", host, "--"] + params.to_flags()
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        m = OFFSET_RE.search(r.stdout + r.stderr)
        return int(m.group(1)) if m else None
    except subprocess.TimeoutExpired:
        return None


def evaluate(servers: list[Server], params: SearchParams, offsets: list[float]) -> float:
    """Test params across all clock offsets and servers, return average error in microseconds."""
    total_err = 0
    count = 0

    for offset_s in offsets:
        expected_us = int(offset_s * 1_000_000)

        # Nudge all servers to this offset
        for srv in servers:
            srv.nudge_time(offset_s)

        # Measure all servers in parallel
        with ThreadPoolExecutor(max_workers=len(servers)) as pool:
            futures = {pool.submit(measure, srv.host, params): srv for srv in servers}
            for fut in futures:
                srv = futures[fut]
                result_us = fut.result()
                if result_us is None:
                    print(f"    {srv.host} offset={offset_s:+.1f}s -> FAIL")
                    return 10_000_000
                err = abs(result_us - expected_us)
                total_err += err
                count += 1
                print(f"    {srv.host} offset={offset_s:+.1f}s -> measured={result_us:+d}us expected={expected_us:+d}us err={err}us")

    avg_err = total_err / count
    print(f"  => avg_err={avg_err:.0f}us")
    return avg_err


def main():
    ap = argparse.ArgumentParser(description="Tune test-http parameters with optuna")
    ap.add_argument("hosts", nargs="+", help="host IPs to test (also used for ssh as root@<ip>)")
    ap.add_argument("--trials", type=int, default=100, help="Number of trials")
    ap.add_argument("--offsets", type=float, nargs="+", default=OFFSETS_S,
                     help="Clock offsets to test (seconds)")
    args = ap.parse_args()

    servers = [Server(host=ip, ssh=f"root@{ip}") for ip in args.hosts]

    def objective(trial: optuna.Trial) -> float:
        params = SearchParams.from_trial(trial)
        return evaluate(servers, params, args.offsets)

    study = optuna.create_study(direction="minimize")
    try:
        study.optimize(objective, n_trials=args.trials)
    except KeyboardInterrupt:
        print("\nInterrupted, showing results so far...")

    for srv in servers:
        srv.nudge_time(0)

    print(f"\n{'='*60}")
    if len(study.trials) > 0 and study.best_trial.value is not None:
        print(f"Best avg error: {study.best_value:.0f}us")
        print(f"Best params: {study.best_params}")
    else:
        print("No completed trials.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
