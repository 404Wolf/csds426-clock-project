# csds426-clock-project

## Recollect data

Full internet ICMP timestamp scan:
```sh
just scan-icmp-clockdiff
```

ICMP-scanned IPs re-probed via HTTP:
```sh
just enrich-http data/icmp_timestamp/icmp_timestamp.csv data/icmp_with_http.csv
```

Tranco top-sites HTTP scan:
```sh
just tranco data/tranco_20k_sample.csv data/tranco_http.csv
```
