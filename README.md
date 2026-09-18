# kahip-rs

Rust bindings and thin wrappers for:
- KaHIP graph partitioning
- KaMinPar graph partitioning

## Upstream projects

- KaHIP: https://github.com/KaHIP/KaHIP
- KaMinPar: https://github.com/KaHIP/KaMinPar

## Local Cargo configuration

This repository supports local path-based wiring via `.cargo/config.toml`.

Example:

```toml
[env]
KAHIP_INCLUDE_DIR = "KAHIP_INCLUDE_DIR"
KAHIP_LIB_DIR = "KAHIP_LIB_DIR"
LD_LIBRARY_PATH = "KAHIP_LIB_DIR"

[target.'cfg(target_os = "linux")']
rustflags = ["-C", "link-arg=-Wl,-rpath,KAHIP_LIB_DIR"]
```

Notes:
- The local `.cargo/config.toml` is gitignored in this repository.
- Keep the paths machine-local.

## API overview

Main module: `src/lib.rs`

### KaHIP API

- `KaHIPGraph<'a>`: graph container alias for KaHIP-compatible index/weight types.
- `KahipMode`: KaHIP partitioner mode (`Fast`, `Eco`, `Strong`, ...).
- `KahipParams`: partition settings with `Default`:
  - `imbalance = 0.03`
  - `suppress_output = true`
  - `seed = 0`
  - `mode = KahipMode::Eco`
- `KaHIPGraph::partition(n_parts, params)` returns `(part, edgecut)`.

### KaMinPar API

- `KaMinParGraph<'a>`: graph container alias for KaMinPar-compatible types.
- `KaminparPreset`: preset selection (`Default`, `Strong`, `TeraPart`, `LargeK`, `VCycle`).
- `KaminparOutputLevel`: output verbosity level.
- `KaminparParams`: partition settings with `Default`:
  - `epsilon = 0.03`
  - `num_threads = available_parallelism()`
  - `preset = KaminparPreset::Default`
  - `output_level = KaminparOutputLevel::Application`
- `KaMinParGraph::partition_with_epsilon(n_parts, params)` returns `(part, edge_cut)`.
