# plato-correlator — Cross-Sense Fusion Engine

Fuse shadow events from multiple sense modules (vision, sonar, motion) into unified assessments with severity classification. When two sensors agree, the confidence goes up. When they disagree, the correlator flags it.

**Part of the [Plato](https://github.com/SuperInstance/plato-shell) ecosystem.**

## What This Gives You

- **Temporal windowing** — correlate events within a configurable time window (default 500ms)
- **Rule-based fusion** — pattern matching with `AND` conditions across sources
- **Priority queue** — critical events bubble to the top
- **Severity classification** — Info/Warning/Alert/Critical based on keywords
- **Automatic expiry** — old shadows are pruned from the window

## Quick Start

```rust
use plato_correlator::{Correlator, ShadowRef, FusionRule, Severity};

let mut correlator = Correlator::new();

// Define a fusion rule
correlator.add_rule(FusionRule::new(
    "vision.person AND sonar.footstep",
    "person_detected",
    0.8,
    Severity::Alert,
));

// Ingest shadows from different sense modules
let vision = ShadowRef::new("vision", "entrance", now, "person detected at door");
let sonar = ShadowRef::new("sonar", "entrance", now, "footstep 2m north");

let fused = correlator.ingest_shadow(vision);
let fused = correlator.ingest_shadow(sonar);

// Check for fused events
let events = correlator.pending_events();
```

## How It Fits

Sits between the sense modules ([plato-vision](https://github.com/SuperInstance/plato-vision), [plato-sonar-text](https://github.com/SuperInstance/plato-sonar-text)) and the rest of the system. The correlator is how multiple sensors become one coherent picture. Used by [plato-shell](https://github.com/SuperInstance/plato-shell) for unified perception.

## Testing

```bash
cargo test
```

## License

MIT
