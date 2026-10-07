# Evaluation and Performance Baselines

Phase 2E adds repeatable correctness and performance checks before the reference behavior is ported to Rust.

## Correctness corpus

`data/fixtures/realistic_corpus.json` contains project-authored synthetic text only. It is not copied from news, Wikipedia articles, books or other third-party prose.

The corpus intentionally mixes:
- CN -> HK and CN -> TW entity + terminology conversion;
- script/character conversion;
- HK/TW character variants;
- phrase exceptions;
- repeated matches in longer sentences;
- no-op safety cases.

Run:

```bash
python scripts/evaluate_corpus.py
```

A correctness run is successful only when every case matches the exact expected output and expected review-needed state.

## Performance benchmark

Run the default local benchmark:

```bash
python scripts/benchmark_localizer.py
```

The report includes:
- input character count;
- warmup and measured iterations;
- deterministic-output check;
- aggregate characters/second;
- mean, p50, p95 and maximum request latency;
- peak Python memory tracked by `tracemalloc` for one localization call.

The Python reference implementation is the behavioral baseline, not the final performance target. Future Rust/Tauri work should use the same corpus and equivalent measurement method, then record environment details alongside results.

## Initial CI sample

The first passing Phase 2E pull-request run used CPython 3.11.17 on a GitHub-hosted Ubuntu 24.04 runner. The smoke payload used the `zh-CN -> zh-TW` route, 5,145 input characters, one warmup and three measured iterations.

Observed sample:
- exact realistic corpus: 8 / 8 cases passed;
- complete unit suite: 44 tests passed;
- deterministic repeated output: yes;
- throughput: about 200,043 characters/second;
- p50 latency: about 24.7 ms;
- p95 latency: about 28.2 ms;
- peak Python allocation tracked by `tracemalloc`: about 1,667 KiB.

These numbers are a historical smoke sample only. Hosted runner hardware and load vary, so they must not be treated as guaranteed product performance.

## CI policy

GitHub-hosted runners are noisy and are not treated as precise benchmark machines. CI therefore uses deliberately generous limits only to detect catastrophic regressions:
- at least 100 characters/second;
- p95 below 10 seconds for the small smoke payload;
- peak tracked Python allocation below 512 MiB.

These thresholds are safety rails, not product targets.

## Comparing implementations

When comparing Python and future Rust implementations:
1. use the same corpus revision;
2. record commit SHA and platform/CPU/RAM;
3. use the same source/target route and comparable payload size;
4. verify exact correctness first;
5. compare warmed p50/p95 latency and throughput;
6. report memory separately from timing because allocation tracking changes timing characteristics.

Do not optimize by weakening deterministic/no-guess behavior or removing provenance output.
