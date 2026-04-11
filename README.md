# cuda-metrics-v2

Enhanced metrics — histograms with percentiles, timers, derivative gauges (Rust)

Part of the Cocapn fleet — a Lucineer vessel component.

## What It Does

### Key Types

- `Bucket` — core data structure
- `Histogram` — core data structure
- `HistogramSnapshot` — core data structure
- `Timer` — core data structure
- `TimerSnapshot` — core data structure
- `DerivativeGauge` — core data structure
- _and 2 more (see source)_

## Quick Start

```bash
# Clone
git clone https://github.com/Lucineer/cuda-metrics-v2.git
cd cuda-metrics-v2

# Build
cargo build

# Run tests
cargo test
```

## Usage

```rust
use cuda_metrics_v2::*;

// See src/lib.rs for full API
// 8 unit tests included
```

### Available Implementations

- `Histogram` — see source for methods
- `Timer` — see source for methods
- `DerivativeGauge` — see source for methods
- `MetricsRegistry` — see source for methods

## Testing

```bash
cargo test
```

8 unit tests covering core functionality.

## Architecture

This crate is part of the **Cocapn Fleet** — a git-native multi-agent ecosystem.

- **Category**: other
- **Language**: Rust
- **Dependencies**: See `Cargo.toml`
- **Status**: Active development

## Related Crates


## Fleet Position

```
Casey (Captain)
├── JetsonClaw1 (Lucineer realm — hardware, low-level systems, fleet infrastructure)
├── Oracle1 (SuperInstance — lighthouse, architecture, consensus)
└── Babel (SuperInstance — multilingual scout)
```

## Contributing

This is a fleet vessel component. Fork it, improve it, push a bottle to `message-in-a-bottle/for-jetsonclaw1/`.

## License

MIT

---

*Built by JetsonClaw1 — part of the Cocapn fleet*
*See [cocapn-fleet-readme](https://github.com/Lucineer/cocapn-fleet-readme) for the full fleet roadmap*
