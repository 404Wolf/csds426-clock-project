import pandas as pd

ECHO = "../data/icmp_echo/icmp_echo.csv"
TIMESTAMP = "../data/icmp_timestamp/icmp_timestamp.csv"
N = 20

print("Loading echo IPs...")
echo = pd.read_csv(ECHO, usecols=["saddr_raw", "success"], dtype={"saddr_raw": "int64", "success": "int8"})
echo_ips = set(echo.loc[echo["success"] == 1, "saddr_raw"])

print("Loading timestamp IPs...")
ts = pd.read_csv(TIMESTAMP, usecols=["saddr", "saddr_raw", "success"], dtype={"saddr_raw": "int64", "success": "int8"})
ts_responding = ts[ts["success"] == 1]

ts_only = ts_responding[~ts_responding["saddr_raw"].isin(echo_ips)]

print(f"\n{len(ts_only):,} timestamp-only rows. First {N} examples:\n")
print(ts_only[["saddr", "saddr_raw"]].drop_duplicates().head(N).to_string(index=False))
