#!/usr/bin/env python3
import argparse
import csv
import sys

def split_csv(path, n):
    with open(path, newline="") as f:
        reader = csv.reader(f)
        header = next(reader)
        chunk_idx = 0
        out = None
        writer = None
        count = 0

        for row in reader:
            if out is None:
                out = open(f"{path}.{chunk_idx:04d}.csv", "w", newline="")
                writer = csv.writer(out)
                writer.writerow(header)
                count = 0
            writer.writerow(row)
            count += 1
            if count == n:
                out.close()
                print(f"wrote {path}.{chunk_idx:04d}.csv ({count} rows)")
                chunk_idx += 1
                out = None

        if out is not None:
            out.close()
            print(f"wrote {path}.{chunk_idx:04d}.csv ({count} rows)")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Split a CSV into chunks of N rows.")
    parser.add_argument("file", help="input CSV file")
    parser.add_argument("n", type=int, help="rows per chunk")
    args = parser.parse_args()
    split_csv(args.file, args.n)
