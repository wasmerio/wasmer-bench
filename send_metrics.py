#!/usr/bin/env python3
import csv
import os
import toml  # pip install toml

# https://github.com/AlekSi/codespeed-client
# pip install codespeed-client
from codespeed_client import Client

BENCHMARK_OUT_DIR = "target/criterion"
BENCHMARKS = ["small_compile",
              "large_compile", "fibonacci", "sha1", "sum", "nbody","fannkuch"]
#BENCHMARKS = ["fannkuch"]
BACKENDS = ["wasmer-clif", "rust-native",
            "wasmer-llvm", "wasmer-dynasm", "wasm-c-api-v8", "wasmi"]
BACKEND_TO_PROJECT = {'wasmer-clif': 'wasmer', 'wasmi': 'wasmi', 'wasmer-dynasm': 'wasmer',
                      'rust-native': 'rust', 'wasmer-llvm': 'wasmer', 'wasm-c-api-v8': 'v8'}

# `git rev-parse HEAD` in the wasm-c-api/v8/v8 directory
V8_COMMIT = "e0ea8246c6ad7b698643995ba25da09d7012f679"

# `rustc +nightly --version`,
# find full commit in https://github.com/rust-lang/rust/commit/{short-id}
RUST_COMMIT = "88f755f8a84df1d9e6b17cf10c96ae8b93481b2e"

# Check commit in https://github.com/paritytech/wasmi
# corresponding to the version in Cargo.toml
WASMI_COMMIT = "0267b20e6ec0085f6dc7d5d813aa2cc17383f9d5"


def main():
    print("Sending benchmarks to metrics API...")
    metrics = []
    for benchmark in BENCHMARKS:
        for backend in BACKENDS:
            metric = get_metric(benchmark, backend)
            if metric is not None:
                metrics.append(metric)
    send_metrics(metrics)


def get_metric(benchmark, backend):
    if backend == "rust-native" and "compile" in benchmark:
        return None
    if backend == "wasmi" and "compile" in benchmark:
        return None
    avg_nanos = get_average_nanos(benchmark, backend)
    if avg_nanos is None:
        return None
    else:
        return {'backend': backend, 'benchmark': benchmark, 'avg_nanos': avg_nanos}


def get_average_nanos(benchmark, backend):
    filename = BENCHMARK_OUT_DIR + "/" + benchmark + "/" + backend + "/new/raw.csv"
    total_nanos = 0.0
    count = 0
    exists = os.path.isfile(filename)
    if exists:
        with open(filename) as csvdatafile:
            reader = csv.DictReader(csvdatafile)
            for row in reader:
                total_nanos += float(row['sample_time_nanos'])
                count += int(row['iteration_count'])
            return total_nanos / count
    else:
        return None


def get_commit_from_cargo_lock(package):
    cargo_lock = toml.load('Cargo.lock')
    for p in cargo_lock['package']:
        if p['name'] == package:
            return p['source'].split("#")[1]
    return None


def get_commit_id(project):
    if project == "wasmer":
        return get_commit_from_cargo_lock("wasmer-runtime-core")
    elif project == "v8":
        return V8_COMMIT
    elif project == "rust":
        return RUST_COMMIT
    elif project == "wasmi":
        return WASMI_COMMIT
    else:
        raise Exception('unknown project: ' + project)


def send_metrics(metrics):
    #print("Sending metrics:")
    # print(metrics)
    client = Client('https://speed.wasmer.io',
                    environment='local-machine-1')
    results = []
    for metric in metrics:
        seconds = metric['avg_nanos'] / 1000000000
        project = BACKEND_TO_PROJECT[metric['backend']]
        commit_id = get_commit_id(project)
        result = {'executable': metric['backend'], 'commitid': commit_id,
                  'benchmark': metric['benchmark'], 'result_value': seconds, 'project': project}
        results.append(result)
        client.add_result(**result)
    print('Sending results:')
    print(results)
    client.upload_results()


if __name__ == '__main__':
    main()
