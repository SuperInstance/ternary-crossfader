# ternary-crossfader

**The art of the blend. Linear, equal-power, S-curve — how two signals become one.**

A crossfader is the simplest and most important mixing tool: one knob that fades between two sources. At position 0, you hear only A. At position 1, only B. In between, a blend. But *how* they blend matters enormously. A linear crossfade (straight line) causes a volume dip in the middle. An equal-power crossfade compensates — the total energy stays constant. An S-curve stays near the edges longer and transitions faster through the center, giving you more time in "pure A" and "pure B" territory.

This crate implements four crossfade curves for ternary signals and provides tools for analyzing what each curve does to the signal's energy, phase, and character.

## What's Inside

- **`FaderCurve`** — enum of curve types: `Linear`, `EqualPower`, `SCurve`, `ConstantPower`
- **`apply(position)`** — compute (left_gain, right_gain) for a position in [0, 1]
- **`crossfade(signal_a, signal_b, position, curve)`** — blend two ternary signals at a given position
- **`auto_crossfade(a, b, curve, steps)`** — sweep from A to B over N steps
- **`spindle_point(curve)`** — the position where left_gain = right_gain (the exact center)

## Quick Example

```rust
use ternary_crossfader::*;

let a = vec![1, 1, 1, 1];
let b = vec![-1, -1, -1, -1];

// Linear crossfade at 50%
let mixed = crossfade(&a, &b, 0.5, FaderCurve::Linear);
// Both at 50% — volume dip in the middle

// Equal-power: constant total energy
let ep = crossfade(&a, &b, 0.5, FaderCurve::EqualPower);
// Cosine-based — no volume dip

// S-curve: stays near the edges longer
let sc = crossfade(&a, &b, 0.5, FaderCurve::SCurve);
// Sigmoid-based — more time in pure A/B territory

// Auto sweep: gradual transition
let sweep = auto_crossfade(&a, &b, FaderCurve::EqualPower, 8);
// 8 steps from pure A to pure B
```

## The Deeper Truth

**The crossfade curve determines the *emotional arc* of the transition.** A linear fade is mechanical — the energy drops in the middle and both signals feel equally weak. An equal-power fade is smooth — the energy stays constant, but both signals lose their identity in the middle. An S-curve is dramatic — it holds onto each signal as long as possible, then switches fast. The choice of curve isn't technical; it's artistic.

In ternary, crossfading has a quantized quality: because the source signals only have three values, the intermediate blends are more like a smooth interpolation between discrete states than a continuous mix. The "in-between" positions create values that aren't strictly ternary — they're *weighted averages* of ternary values. The output must be snapped back to {-1, 0, +1} at each step, which creates an interesting staircase effect as the blend progresses.

**Use cases:**
- **DJ mixing** — crossfade between tracks with different curve feels
- **Film/game audio** — transition between ambient soundscapes
- **Live performance** — blend between patches in real-time
- **Generative music** — algorithmic transitions between sections
- **Education** — the simplest signal blending operation

## See Also

- **ternary-mixer** — mixing multiple sources (crossfading is 2-channel mixing)
- **ternary-pan** — panning is a spatial crossfade (left ↔ right)
- **ternary-wave** — generate the signals you're crossfading
- **ternary-envelope** — envelopes shape the crossfade over time
- **ternary-rack** — wire crossfaders into a modular signal chain

## Install

```bash
cargo add ternary-crossfader
```

## License

MIT
