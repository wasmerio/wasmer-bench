#!/usr/bin/env python3
import csv
import os
import platform
import toml  # pip install toml
import statistics

# https://github.com/AlekSi/codespeed-client
# pip install codespeed-client
from codespeed_client import Client

BENCHMARK_OUT_DIR = "target/criterion"
BENCHMARKS = ["small_compile",
              "large_compile", "fibonacci", "sha1", "sum", "nbody", "fannkuch",
              "basic dynfunc", "basic static func", "basic dynfunc with many args", "basic static func with many args"]
# BENCHMARKS = ["fannkuch"]
BACKENDS = ["wasmer-clif", "rust-native",
            "wasmer-llvm", "wasmer-dynasm", "wasm-c-api-v8", "wasmi"]
BACKEND_TO_PROJECT = {'wasmer-clif': 'wasmer', 'wasmi': 'wasmi', 'wasmer-dynasm': 'wasmer',
                      'rust-native': 'rust', 'wasmer-llvm': 'wasmer', 'wasm-c-api-v8': 'v8'}

LEGACY_NAME_TO_NAME = {'wasmer-clif': 'cranelift', 'wasmi': 'wasmi', 'wasmer-dynasm': 'singlepass',
                      'rust-native': 'rust-native', 'wasmer-llvm': 'llvm', 'wasm-c-api-v8': 'wasm-c-api-v8'}

# `git rev-parse HEAD` in the wasm-c-api/v8/v8 directory
V8_COMMIT = "e0ea8246c6ad7b698643995ba25da09d7012f679"

# `rustc +nightly --version`,
# find full commit in https://github.com/rust-lang/rust/commit/{short-id}
RUST_COMMIT = "3c3d3c1777041200bb7ed7a65b6562d62899778c"

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
                print(metric)
                metrics.append(metric)
    send_metrics(metrics)


def get_metric(benchmark, backend):
    if backend == "rust-native" and "compile" in benchmark:
        return None
    if backend == "wasmi" and "compile" in benchmark:
        return None
    stats_nanos = get_stats_nanos(benchmark, backend)
    if stats_nanos is None:
        return None
    else:
        return {'backend': backend, 'benchmark': benchmark, 'stats': stats_nanos}


def get_stats_nanos(benchmark, backend):
    backend_name = LEGACY_NAME_TO_NAME[backend]
    filename = BENCHMARK_OUT_DIR + "/" + benchmark + " " + backend_name + "/new/raw.csv"
    total_nanos = 0.0
    count = 0
    exists = os.path.isfile(filename)
    if exists:
        min = None
        max = None
        with open(filename) as csvdatafile:
            reader = csv.DictReader(csvdatafile)
            data = []
            for row in reader:
                total_nanos = float(row['sample_measured_value'])
                assert (row['unit'] == 'ns'),"measured value is not in nanoseconds! unhandeled case!"
                iters = int(row['iteration_count'])
                nanos = total_nanos / iters
                if min is None or nanos < min:
                    min = nanos
                if max is None or nanos > max:
                    max = nanos
                data.append(nanos)
            stdev = statistics.stdev(data)
            mean = statistics.mean(data)
            return {'average': mean, 'min': min, 'max': max, 'stdev': stdev}
    else:
        return None


def get_commit_from_cargo_lock(package):
    cargo_lock = toml.load('Cargo.lock')
    for p in cargo_lock['package']:
        if p['name'] == package:
            return p['source'].split("#")[1]
    return None

def running_on_github_actions():
    return os.environ.get('GITHUB_ACTIONS') == 'true'

def get_commit_id(project):
    if running_on_github_actions():
        return os.environ.get('GITHUB_SHA')
    elif project == "wasmer":
        return get_commit_from_cargo_lock("wasmer-vm")
    elif project == "v8":
        return V8_COMMIT
    elif project == "rust":
        return RUST_COMMIT
    elif project == "wasmi":
        return WASMI_COMMIT
    else:
        raise Exception('unknown project: ' + project)

# Used for reporting where the benchmark was run
def get_environment_name():
    ## The network name of this computer
    node_name = platform.node()
    #system = platform.system()
    #architecture = platform.machine()
    #release = platform.release()

    #system_name = "{node_name} {architecture} {system} {release}".format(
    #    node_name=node_name,
    #    architecture=architecture,
    #    system=system,
    #    release=release)

    if running_on_github_actions():
        #return "GITHUB ACTIONS: " + system_name
        # temp name for testing
        return "GITHUB_ACTIONS_TEST"
    else:
        return node_name

## Extra information about the enivronment
#def env_metadata():
#    if os.environ.get('GITHUB_ACTIONS') == 'true':
#        # WORKFLOW is the name of the workflow
#        workflow_name = os.environ.get('GITHUB_WORKFLOW')
#        # RUN_ID is the unique ID per Action run
#        # RUN_ID does not change if the Action is re-run
#        run_id = os.environ.get('GITHUB_RUN_ID')
#        # RUN NUMBER is the number of times it has been run
#        # This number disambiguates which re-run of the Action this is
#        run_number = os.environ.get('GITHUB_RUN_NUMBER')
#
#        return str({'workflow_name': workflow_name,
#                    'run_id': run_id,
#                    'run_number': run_number})


def send_metrics(metrics):
    # print("Sending metrics:")
    # print(metrics)
    environment = get_environment_name()
    client = Client('https://speed.wasmer.io',
                    environment=environment)
    results = []
    for metric in metrics:
        stats = metric['stats']
        seconds = stats['average'] / 1000000000
        min = stats['min'] / 1000000000
        max = stats['max'] / 1000000000
        stdev = stats['stdev'] / 1000000000
        project = BACKEND_TO_PROJECT[metric['backend']]
        commit_id = get_commit_id(project)

        result = {'executable': metric['backend'], 'commitid': commit_id,
                  'min': min, 'max': max, 'std_dev': stdev,
                  'benchmark': metric['benchmark'], 'result_value': seconds, 'project': project}
        results.append(result)
        client.add_result(**result)
    print('Sending results:')
    print(results)
    client.upload_results()


if __name__ == '__main__':
    main()
