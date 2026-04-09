#!/usr/bin/env python3
import argparse
import sys

def split_csv(path, n):
    with open(path, "rb") as f:
        header = f.readline()
        chunk_idx = 0
        count = 0
        out = None

        for line in f:
            if out is None:
                out = open(f"{path}.{chunk_idx:04d}.csv", "wb")
                out.write(header)
                count = 0
            out.write(line)
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
