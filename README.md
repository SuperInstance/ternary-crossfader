# ternary-crossfader

**Smooth blending between ternary channels.** DJ crossfader dynamics for agent populations — two signals enter, one blended signal leaves.

## Why This Exists

In a multi-agent ternary system, you don't always want hard handoffs. When one agent fades out and another takes over, the transition matters. A sudden switch from a contrarian (-1) to an agreeable (+1) agent creates discontinuities that cascade through the population.

Real DJs solved this decades ago: crossfaders. The question is how you blend two ternary streams when the values are {-1, 0, +1} — you can't just average them naively without understanding the energy characteristics of each curve.

This crate implements four crossfader curves, each with different energy properties, plus utilities for finding equilibrium points in blended signals and preventing clipping in the output.

## The Physics Behind It

### Why Four Curves?

Not all crossfaders are created equal. In audio engineering, the choice of curve determines what happens at the center position:

- **Linear** — the obvious choice: left_gain = 1-p, right_gain = p. Problem: at center, both channels are at 50%, so the total energy drops. Two +1 signals blending at 50% each give 1.0, but a +1 and a -1 cancel to 0. This creates a "dip" in the middle.

- **Equal Power** — uses cosine curves so that the sum of squares remains constant. This prevents the center dip. Mathematically: `left = cos(π/2 · (1-p))`, `right = cos(π/2 · p)`. The squared magnitudes always sum to 1.0.

- **S-Curve** — steep transition at center, gentle at extremes. Uses a sigmoid: `1 / (1 + e^(-6(p - 0.5)))`. Most of the transition happens in a narrow band around 0.5. Good for situations where you want to keep both agents distinct until the last moment.

- **Constant Power** — logarithmic curve that preserves perceived loudness. Uses `exp(0.5 · ln(p))` which gives a different power profile than equal-power.

### The Spindle

The `find_spindle` function locates the equilibrium point in a ternary stream — the position where local energy is minimized. This is the 8-ball spot: where the system wants to rest. In practice, this is where a blended signal settles when two agents with opposing stances find their compromise position.

### Connection to RPS Dynamics

When two agents interact through Rock-Paper-Scissors dominance (-1 beats +1, +1 beats 0, 0 beats -1), the crossfader determines how quickly one agent's influence supplants the other's. A linear crossfade means the transition is gradual and predictable. An S-curve means both agents hold their ground until a sudden flip — matching the sudden dominance reversals seen in the RPS experiments with period ~50.

## Key Types and Functions

```rust
/// Crossfader curve type.
pub enum FaderCurve { Linear, EqualPower, SCurve, ConstantPower }

impl FaderCurve {
    /// Apply crossfader curve. position in [0,1], returns (left_gain, right_gain).
    pub fn apply(&self, position: f64) -> (f64, f64)
}

/// Blend two ternary streams via crossfader.
pub fn crossfade(left: &[i8], right: &[i8], position: f64, curve: FaderCurve) -> Vec<f64>

/// Hard cut — instant switch at a threshold position.
pub fn hard_cut(left: &[i8], right: &[i8], position: f64, threshold: f64) -> Vec<i8>

/// Transform mixing — scratch-style back-and-forth at given rate.
pub fn transform_mix(left: &[i8], right: &[i8], rate: f64, ticks: usize) -> Vec<Vec<i8>>

/// Find the equilibrium point (spindle) in a ternary stream.
pub fn find_spindle(stream: &[i8]) -> (usize, f64)

/// Balance point — center of mass of ternary distribution.
pub fn balance_point(values: &[i8]) -> f64

/// Channel gain staging — prevent clipping while preserving dynamics.
pub fn gain_stage(channel: &mut [f64], target_peak: f64) -> f64
```

## Usage

```rust
use ternary_crossfader::{crossfade, hard_cut, FaderCurve, find_spindle, gain_stage};

let agent_a = vec![1, 1, 0, -1, -1];  // agreeing → reflecting → contrarian
let agent_b = vec![-1, 0, 1, 1, 1];   // contrarian → reflecting → agreeing

// Smooth blend at center position
let blended = crossfade(&agent_a, &agent_b, 0.5, FaderCurve::EqualPower);
// Energy preserved — no center dip

// Hard cut: switch at 70% through
let cut = hard_cut(&agent_a, &agent_b, 0.7, 0.7);

// Find where the signal is calmest
let (spindle_pos, spindle_energy) = find_spindle(&blended_as_i8);

// Prevent clipping after mixing
let mut mixed = vec![1.5, -0.8, 2.1];
let applied_gain = gain_stage(&mut mixed, 1.0);
```

### Transform Mixing (Scratch)

```rust
use ternary_crossfader::transform_mix;

// Scratch between two agents — they alternate based on a sine rate
let results = transform_mix(&agent_a, &agent_b, 1.0, 20);
// Returns 20 vectors, each a snapshot of which agent is "on"
```

## In the Ternary Fleet

This is the **transition layer** in the DJ metaphor product stack:

- `ternary-tenforward` — conversation engine producing the agent streams
- `ternary-tempo` — BPM estimation determines *when* to crossfade
- **ternary-crossfader** — *how* to blend during transitions
- `ternary-mixer` — multi-channel mixing when you have more than two agents
- `ternary-envelope` — ADSR shaping of individual agent contributions

## References

- Equal-power crossfade theory: the cosine curve ensures `left² + right² = 1` at all positions
- RPS wave experiments: crossfade rate maps to dominance transition speed in population dynamics
- Fibonacci period 8: the spindle tends to appear at positions aligned with the natural rhythm

## License

MIT
