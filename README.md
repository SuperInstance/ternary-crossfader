# Ternary Crossfader

Crossfader dynamics for **ternary channel blending** — smooth interpolation, hard cut mixing, and transform weighting between {-1, 0, +1} signal streams. Provides DJ-style crossfader curves (linear, equal-power, S-curve, constant-power), spindle detection for equilibrium finding, and gain staging for clipping prevention.

## Why It Matters

When two ternary agent fleets produce competing signal streams, you need to blend them. The crossfader is the mathematical tool for this — borrowed from audio engineering but applied to ternary logic. The position parameter $p \in [0, 1]$ controls the blend ratio:

- $p = 0$: pure left (stream A)
- $p = 1$: pure right (stream B)
- $p = 0.5$: equal blend

The choice of crossfader curve dramatically affects the result. A linear curve at center produces amplitude 0.5+0.5 = 1.0, but an **equal-power** curve uses trigonometric weighting that preserves power spectral density across the blend.

## How It Works

### Crossfader Curves

Given position $p \in [0, 1]$:

**Linear**:
$$L = 1 - p, \quad R = p$$

**Equal-Power** (constant total power):
$$L = \cos\left(\frac{\pi}{2}(1-p)\right), \quad R = \cos\left(\frac{\pi}{2} p\right)$$

Note: $L^2 + R^2 = 1$ for all $p$ — this is the constant-power property, derived from $\cos^2\theta + \sin^2\theta = 1$.

**S-Curve** (sigmoid):
$$s = \frac{1}{1 + e^{-6(p - 0.5)}}, \quad L = 1 - s, \quad R = s$$

This produces a steep transition at $p = 0.5$ with a dead zone at the extremes — useful for scratch-style mixing where you want sharp cuts.

**Constant-Power** (geometric):
$$L = e^{0.5 \ln(1-p)}, \quad R = e^{0.5 \ln p}$$

### Spindle Detection

The "spindle" is the equilibrium point — the index in a ternary stream where local energy is minimized:

$$E(i) = \sum_{j \in N(i)} |x_j - x_i|$$

The spindle is $\arg\min_i E(i)$. It identifies where the stream is most settled — the natural resting point.

### Transform Mixing

Transform mixing creates a back-and-forth pattern using a sinusoidal gate:

$$\text{gate}(t) = \frac{\sin(2\pi r \cdot t) + 1}{2}$$

$$\text{output}_t = \begin{cases} \text{left} & \text{gate}(t) < 0.5 \\ \text{right} & \text{gate}(t) \geq 0.5 \end{cases}$$

This creates rhythmic alternation at rate $r$ — simulating DJ "transform" scratching.

### Gain Staging

Normalizes a channel to a target peak amplitude:

$$g = \frac{t}{\max_i |x_i|}$$

Applied as $x_i \leftarrow g \cdot x_i$ for all $i$. Prevents clipping while preserving dynamics.

### Complexity

| Operation | Time |
|-----------|------|
| `FaderCurve::apply(p)` | O(1) |
| `crossfade(left, right, p, curve)` | O(N) |
| `hard_cut(left, right, p, threshold)` | O(N) |
| `transform_mix(left, right, rate, ticks)` | O(N · T) |
| `find_spindle(stream)` | O(N · W) where W = window |
| `gain_stage(channel, target)` | O(N) |

## Quick Start

```rust
use ternary_crossfader::{crossfade, hard_cut, FaderCurve, find_spindle, gain_stage};

let left:  &[i8] = &[1, 1, 1, -1, 0];
let right: &[i8] = &[-1, -1, -1, 1, 0];

// Smooth blend at center position
let blended = crossfade(left, right, 0.5, FaderCurve::Linear);
assert!(blended.iter().all(|v| v.abs() < 0.01)); // cancels at center

// Equal-power blend
let ep = crossfade(left, right, 0.3, FaderCurve::EqualPower);

// Hard cut
let cut = hard_cut(left, right, 0.7, 0.5);
assert_eq!(cut, right[..5].to_vec());

// Find equilibrium
let stream = vec![1, 1, 0, 0, 0, 1, 1];
let (pos, energy) = find_spindle(&stream);
assert_eq!(stream[pos], 0); // spindle at a zero

// Gain stage a channel
let mut channel = vec![2.0, -2.0, 1.0];
gain_stage(&mut channel, 1.0); // normalizes to peak 1.0
```

## API

### Crossfader

| Function | Description |
|----------|-------------|
| `crossfade(left, right, position, curve) → Vec<f64>` | Blend two ternary streams |
| `hard_cut(left, right, position, threshold) → Vec<i8>` | Instant switch at threshold |
| `transform_mix(left, right, rate, ticks) → Vec<Vec<i8>>` | Rhythmic alternation pattern |

### Analysis

| Function | Description |
|----------|-------------|
| `find_spindle(stream) → (usize, f64)` | Minimum-energy equilibrium point |
| `balance_point(values) → f64` | Center of mass (mean of values) |
| `gain_stage(channel, target_peak) → f64` | Normalize and return gain applied |

### FaderCurve

| Variant | Property |
|---------|----------|
| `Linear` | $L + R = 1$, simple amplitude pan |
| `EqualPower` | $L^2 + R^2 = 1$, constant total power |
| `SCurve` | Sigmoid — steep at center, flat at extremes |
| `ConstantPower` | Geometric — preserves energy in log domain |

## Architecture Notes

The crossfader implements the **γ + η = C** conservation link through the equal-power invariant:

- **γ (structure)**: the two input streams being blended — fixed identities
- **η (dynamics)**: the crossfader position that perturbs the blend ratio over time
- **C (conservation)**: the constant-power invariant $L^2 + R^2 = 1$ — no matter where the fader sits, total energy is preserved

The spindle detector finds the **η-minimum** — the point where perturbation energy is lowest. This is the ternary equivalent of finding the "zero crossing" in audio — the safest point for a cut or transition because there's no discontinuity.

## References

| Bristow-Johnson, R. (1995). *Equal-Intensity Stereo Panning*. Audio Engineering Society Convention.
| Puckette, M. (2007). *The Theory and Technique of Electronic Music*. World Scientific — crossfading and spectral techniques.
| Roads, C. (1996). *The Computer Music Tutorial*. MIT Press — amplitude mixing and gain staging.
| Zölzer, U. (2008). *Digital Audio Signal Processing* (2nd ed.). Wiley — constant-power panning laws.

## License: MIT
