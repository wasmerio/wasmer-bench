#!/usr/bin/env python3
import csv
import os

BENCHMARK_OUT_DIR = "target/criterion"
BENCHMARKS = ["small compile benchmark",
              "large compile benchmark", "fib 30", "sha1", "sum 1, 2", "nbody"]
BACKENDS = ["clif", "native", "llvm"]

METRICS_API_KEY = os.environ['METRICS_API_KEY']
METRICS_HOST = os.environ['METRICS_HOST']


def main():
    print("Sending benchmarks to metrics API...")
    for benchmark in BENCHMARKS:
        for backend in BACKENDS:
            report_metric(benchmark, backend)


def report_metric(benchmark, backend):
    if backend == "native" and "compile" in benchmark:
        return
    avg_nanos = get_average_nanos(benchmark, backend)
    tags = {'backend': backend}
    send_metric(benchmark, avg_nanos, tags)


def get_average_nanos(benchmark, backend):
    filename = BENCHMARK_OUT_DIR + "/" + benchmark + "/" + backend + "/new/raw.csv"
    total_nanos = 0.0
    count = 0
    with open(filename) as csvdatafile:
        reader = csv.DictReader(csvdatafile)
        for row in reader:
            total_nanos += float(row['sample_time_nanos'])
            count += int(row['iteration_count'])
    return total_nanos / count


def send_metric(name, value_nanos, tags):
    print("Sending metric:")
    print(name)
#     print(value)
#     print(tags)
    metric = get_metric_formatted(name, value_nanos, tags)
    import socket
    conn = socket.create_connection(
        (METRICS_HOST, 2003))
    conn.send(metric.encode())
    conn.close()
#     print(metric)


def get_metric_formatted(name, value_nanos, tags):
    unit = "nanos" if "sum" in name else "micros"
    value = value_nanos if "sum" in name else value_nanos / 1000
    metric_name = name.replace(" ", "_").replace(",", "_") + "_avg_" + unit
    metric = ".benches." + metric_name
    tags_str = []
    metric = metric + ";"
    for k, v in tags.items():
        tags_str.append(k + "=" + v)
    metric = metric + ";".join(tags_str)
    metric = metric + " " + str(value) + "\n"
    print(metric)
    metric = METRICS_API_KEY + metric
    return metric


if __name__ == '__main__':
    main()
