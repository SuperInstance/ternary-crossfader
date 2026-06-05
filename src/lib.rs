#![forbid(unsafe_code)]
//! Crossfader dynamics — blending, cutting, and transforming ternary channels.

/// Crossfader curve type.
#[derive(Debug, Clone, Copy)]
pub enum FaderCurve { Linear, EqualPower, SCurve, ConstantPower }

impl FaderCurve {
    /// Apply crossfader curve. position in [0,1], returns (left_gain, right_gain).
    pub fn apply(&self, position: f64) -> (f64, f64) {
        let p = position.clamp(0.0, 1.0);
        match self {
            FaderCurve::Linear => (1.0 - p, p),
            FaderCurve::EqualPower => {
                let left = (0.5 * std::f64::consts::PI * (1.0 - p)).cos();
                let right = (0.5 * std::f64::consts::PI * p).cos();
                (left, right)
            }
            FaderCurve::SCurve => {
                let s = 1.0 / (1.0 + (-6.0 * (p - 0.5)).exp());
                (1.0 - s, s)
            }
            FaderCurve::ConstantPower => {
                let left = ((1.0 - p).max(0.001).ln() * 0.5).exp();
                let right = (p.max(0.001).ln() * 0.5).exp();
                (left, right)
            }
        }
    }
}

/// Blend two ternary streams via crossfader.
pub fn crossfade(left: &[i8], right: &[i8], position: f64, curve: FaderCurve) -> Vec<f64> {
    let (lg, rg) = curve.apply(position);
    let len = left.len().min(right.len());
    (0..len).map(|i| left[i] as f64 * lg + right[i] as f64 * rg).collect()
}

/// Hard cut — instant switch at a threshold position.
pub fn hard_cut(left: &[i8], right: &[i8], position: f64, threshold: f64) -> Vec<i8> {
    let len = left.len().min(right.len());
    if position < threshold { left[..len].to_vec() } else { right[..len].to_vec() }
}

/// Transform mixing — scratch-style back-and-forth at given rate.
pub fn transform_mix(left: &[i8], right: &[i8], rate: f64, ticks: usize) -> Vec<Vec<i8>> {
    let len = left.len().min(right.len());
    (0..ticks).map(|t| {
        let phase = ((t as f64 * rate).sin() + 1.0) / 2.0;
        if phase < 0.5 { left[..len].to_vec() } else { right[..len].to_vec() }
    }).collect()
}

/// Spindle detection — find the equilibrium point in a ternary stream.
/// The spindle is where the 8-ball sits still — minimum energy.
pub fn find_spindle(stream: &[i8]) -> (usize, f64) {
    let mut best_pos = 0;
    let mut best_energy = f64::MAX;
    for i in 0..stream.len() {
        // Energy = sum of |value - center| for neighborhood
        let mut energy = 0.0;
        for j in stream.len().saturating_sub(5)..stream.len().min(i + 6) {
            energy += (stream[j] - stream[i]).abs() as f64;
        }
        if energy < best_energy {
            best_energy = energy;
            best_pos = i;
        }
    }
    (best_pos, best_energy)
}

/// Balance point — center of mass of ternary distribution.
pub fn balance_point(values: &[i8]) -> f64 {
    if values.is_empty() { return 0.0; }
    values.iter().map(|&v| v as f64).sum::<f64>() / values.len() as f64
}

/// Channel gain staging — prevent clipping while preserving dynamics.
pub fn gain_stage(channel: &mut [f64], target_peak: f64) -> f64 {
    let peak = channel.iter().map(|v| v.abs()).fold(0.0f64, f64::max);
    if peak > 0.0 {
        let gain = target_peak / peak;
        for v in channel.iter_mut() { *v *= gain; }
        gain
    } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_linear_center() { let (l, r) = FaderCurve::Linear.apply(0.5); assert!((l - 0.5).abs() < 0.01); assert!((r - 0.5).abs() < 0.01); }
    #[test] fn test_linear_left() { let (l, r) = FaderCurve::Linear.apply(0.0); assert!((l - 1.0).abs() < 0.01); assert!((r - 0.0).abs() < 0.01); }
    #[test] fn test_linear_right() { let (l, r) = FaderCurve::Linear.apply(1.0); assert!((l - 0.0).abs() < 0.01); assert!((r - 1.0).abs() < 0.01); }
    #[test] fn test_equal_power_symmetry() { let (l1, r1) = FaderCurve::EqualPower.apply(0.3); let (l2, r2) = FaderCurve::EqualPower.apply(0.7); assert!((l1 - r2).abs() < 0.01); }
    #[test] fn test_s_curve_sharp() { let (_, r1) = FaderCurve::SCurve.apply(0.4); let (_, r2) = FaderCurve::SCurve.apply(0.6); assert!(r2 - r1 > 0.3, "S-curve should be steep at center"); }
    #[test] fn test_crossfade_blend() { let result = crossfade(&[1,1,1], &[-1,-1,-1], 0.5, FaderCurve::Linear); assert!(result.iter().all(|v| v.abs() < 0.01)); }
    #[test] fn test_crossfade_left() { let result = crossfade(&[1], &[-1], 0.0, FaderCurve::Linear); assert!((result[0] - 1.0).abs() < 0.01); }
    #[test] fn test_hard_cut_left() { let result = hard_cut(&[1,1], &[-1,-1], 0.3, 0.5); assert_eq!(result, vec![1,1]); }
    #[test] fn test_hard_cut_right() { let result = hard_cut(&[1,1], &[-1,-1], 0.7, 0.5); assert_eq!(result, vec![-1,-1]); }
    #[test] fn test_transform_mix() { let results = transform_mix(&[1,1], &[-1,-1], 1.0, 10); assert_eq!(results.len(), 10); }
    #[test] fn test_transform_alternates() { let results = transform_mix(&[1], &[-1], 0.5, 20); let switches = results.windows(2).filter(|w| w[0] != w[1]).count(); assert!(switches > 0); }
    #[test] fn test_find_spindle() { let stream = vec![1,1,0,0,0,-1,-1]; let (pos, energy) = find_spindle(&stream); assert!(energy >= 0.0); assert!(pos < stream.len()); }
    #[test] fn test_spindle_at_zeros() { let stream = vec![1,1,0,0,0,1,1]; let (pos, _) = find_spindle(&stream); assert_eq!(stream[pos], 0); }
    #[test] fn test_balance_point() { assert!((balance_point(&[1,-1,0]) - 0.0).abs() < 0.01); }
    #[test] fn test_balance_positive() { assert!(balance_point(&[1,1,1]) > 0.0); }
    #[test] fn test_gain_stage() { let mut ch = vec![2.0, -2.0, 1.0]; let g = gain_stage(&mut ch, 1.0); assert!(g < 1.0); assert!(ch.iter().map(|v| v.abs()).fold(0.0f64, f64::max) <= 1.01); }
    #[test] fn test_gain_stage_quiet() { let mut ch = vec![0.1, 0.2]; let g = gain_stage(&mut ch, 1.0); assert!(g > 1.0); }
    #[test] fn test_gain_stage_silent() { let mut ch = vec![0.0, 0.0]; let g = gain_stage(&mut ch, 1.0); assert_eq!(g, 1.0); }
}
