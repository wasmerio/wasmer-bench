#!/usr/bin/env python3
import csv
import os
import statistics

BENCHMARK_OUT_DIR = "target/criterion"
BENCHMARKS = ["small_compile",
              "large_compile", "fibonacci", "sha1", "sum", "nbody", "fannkuch"]
BACKENDS = ["wasmer-clif", "rust-native",
            "wasmer-llvm", "wasmer-dynasm", "wasm-c-api-v8", "wasmi"]


def main():
    native = get_native_nanos()
    out = '%-24s%-12s%-12s%-12s' % ('benchmark',
                                    'backend', 'avg nanos', 'native ratio')
    print(out)
    for benchmark in BENCHMARKS:
        for backend in BACKENDS:
            report_metric(benchmark, backend, native[benchmark])
        print("\n")


def get_native_nanos():
    native = {}
    for benchmark in BENCHMARKS:
        if "compile" in benchmark:
            native[benchmark] = None
        else:
            native[benchmark] = get_average_nanos(benchmark, "rust-native")
    return native


def report_metric(benchmark, backend, native_nanos):
    if backend == "rust-native" and "compile" in benchmark:
        return
    if backend == "wasmi" and "compile" in benchmark:
        return
    if backend == "v8" and "compile" in benchmark:
        return
    avg_nanos = get_average_nanos(benchmark, backend)
    tags = {'backend': backend}
    if avg_nanos is not None:
        ratio = "N/A" if native_nanos is None else '%0.2f' % (
            avg_nanos / native_nanos)
        out = '%-24s%-14s%-14i%-12s' % (benchmark,
                                        backend, avg_nanos, ratio)
    else:
        out = '%-24s%-14s%-14s%-12s' % (benchmark,
                                        backend, "--", "--")
    print(out)


def get_average_nanos(benchmark, backend):
    filename = BENCHMARK_OUT_DIR + "/" + benchmark + "/" + backend + "/new/raw.csv"
    total_nanos = 0.0
    count = 0
    exists = os.path.isfile(filename)
    if exists:
        data = []
        with open(filename) as csvdatafile:
            reader = csv.DictReader(csvdatafile)
            for row in reader:
                total_nanos = float(row['sample_time_nanos'])
                iters = int(row['iteration_count'])
                data.append(total_nanos / iters)
        return statistics.mean(data)
    else:
        return None


if __name__ == '__main__':
    main()
