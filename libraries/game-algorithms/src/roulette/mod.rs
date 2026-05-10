use crate::common::{chacha::generate_numbers, seed_mixer::SeedMixableNumber};

// CN: 轮盘可落点数量（0..=36）。
// EN: Count of roulette wheel pockets (0..=36).
pub const ROULETTE_POCKET_COUNT: u8 = 37;

// CN: 基于根种子与轮次因子生成可复验的中奖号码。
// EN: Generates a reproducible winning number from root seed and round factor.
pub fn generate_winning_number<T: SeedMixableNumber>(seed: [u8; 32], round: T) -> u8 {
  let mixed_seed = round.mix_into_seed(seed);
  generate_numbers(mixed_seed, 0u8, ROULETTE_POCKET_COUNT - 1, 1)
    .into_iter()
    .next()
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_generate_winning_number_is_in_range() {
    let winning_number = generate_winning_number([7u8; 32], 42u64);
    assert!(winning_number <= 36);
  }

  #[test]
  fn test_generate_winning_number_is_deterministic() {
    let seed = [9u8; 32];
    let round = 123u64;

    let first = generate_winning_number(seed, round);
    let second = generate_winning_number(seed, round);

    assert_eq!(first, second);
  }

  #[test]
  fn test_generate_winning_number_changes_with_round() {
    let seed = [11u8; 32];

    let round_1 = generate_winning_number(seed, 1u64);
    let round_2 = generate_winning_number(seed, 2u64);

    assert_ne!(round_1, round_2);
  }
}
