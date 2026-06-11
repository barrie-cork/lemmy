//! Sandbox helpers used to dogfood the `/auto-phase` orchestration pipeline.
//! Not part of any governance feature — safe to revert.

/// Clamp `value` to at most `max`, returning the smaller of the two.
///
/// Pure, dependency-free, side-effect-free. Exists only to exercise the
/// full plan → impl → validate → PR → merge pipeline on a minimal change.
pub fn sandbox_clamp(value: u32, max: u32) -> u32 {
  value.min(max)
}

#[cfg(test)]
mod tests {
  use super::sandbox_clamp;

  #[test]
  fn clamps_above_max() {
    assert_eq!(sandbox_clamp(10, 5), 5);
  }

  #[test]
  fn passes_through_below_max() {
    assert_eq!(sandbox_clamp(3, 5), 3);
  }
}
