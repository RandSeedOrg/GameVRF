use crate::common::{chacha::generate_unique_numbers, seed_mixer::SeedMixableNumber};

/// Generates a sequence of deterministic random numbers (balls) for a Keno game round.
/// 为 Keno（基诺）游戏的某个特定轮次产生一组具有可确定性的随机数字（开奖球）。
///
/// This function relies on a cryptographic approach by taking a 32-byte root seed
/// and mixing it with a dynamic factor (such as the `round` number) to derive a unique seed.
/// It then uses the underlying ChaCha20 PRNG to yield exactly `count` distinct numbers
/// within the inclusive range [`min`, `max`].
/// 该函数依赖于密码学安全的机制。它将 32 字节的根种子与动态因子（例如：`round` 轮次/期号）进行混合，
/// 派生出该局唯一的随机种子，随后运用底层的 ChaCha20 伪随机数发生器，
/// 在闭区间 [`min`, `max`] 内毫无重复地抽出且仅抽出 `count` 个数字。
pub fn generate_numbers<T: SeedMixableNumber>(seed: [u8; 32], round: T, count: usize, min: u8, max: u8) -> Vec<u8> {
  // Step 1: Mix the dynamic factor into the seed to secure uniqueness for the current outcome.
  // 步骤 1: 将动态因子通过或异或混入原始种子并通过Chacha20RNG处理，从而保障当前轮次抽签结果的独立性和唯一性。
  let mixed_seed = round.mix_into_seed(seed);

  // Step 2: Use the PRNG bounded collection generator to safely yield the unique random set.
  // 步骤 2: 调用被扩展的随机抽取器，在给定的上下限数组内安全地生成不重复集合。
  generate_unique_numbers(mixed_seed, min, max, count)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;

  #[test]
  fn test_generate_numbers() {
    let seed = [3u8; 32];
    let round = 1;
    let min = 1u8;
    let max = 40u8;
    let count = 20;

    let numbers = generate_numbers(seed, round, count, min, max);

    assert_eq!(numbers.len(), count);
    for &num in &numbers {
      assert!(num >= min && num <= max, "Number {} out of bounds", num);
    }
  }

  #[test]
  fn test_generate_numbers_are_unique() {
    let seed = [5u8; 32];
    let round = 1;
    let min = 1u8;
    let max = 40u8;
    let count = 20;

    let numbers = generate_numbers(seed, round, count, min, max);

    assert_eq!(numbers.iter().copied().collect::<HashSet<_>>().len(), count);
  }

  #[test]
  fn test_generate_numbers_diff_rounds_diff_results() {
    let seed = [4u8; 32];
    let min = 1u8;
    let max = 40u8;
    let count = 20;

    let numbers_round1 = generate_numbers(seed, 1, count, min, max);
    let numbers_round2 = generate_numbers(seed, 2, count, min, max);

    assert_ne!(numbers_round1, numbers_round2, "Different rounds should produce different number patterns");
  }
}
