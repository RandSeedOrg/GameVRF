use rand::SeedableRng;
use rand::{distr::uniform::SampleUniform, Rng};
use rand_chacha::ChaCha20Rng;

/// 中文：为整数类型提供基于 ChaCha20 的确定性随机数生成接口。
/// EN: Provides a deterministic ChaCha20-based random number generation interface for integer types.
pub trait ChaCha20RNGGenerator {
  /// 中文：使用固定种子在闭区间 [min, max] 内生成 count 个可复现的随机值。
  /// EN: Generates count reproducible random values within the inclusive range [min, max] from a fixed seed.
  fn chacha20_rand(seed: [u8; 32], min: Self, max: Self, count: usize) -> Vec<Self>
  where
    Self: Sized;
}

/// 中文：在指定 u8 范围内生成不重复的随机数，返回结果的顺序由种子决定且可复现。
/// EN: Generates unique random numbers within the given u8 range, with a seed-determined and reproducible order.
pub fn generate_unique_numbers(seed: [u8; 32], min: u8, max: u8, count: usize) -> Vec<u8> {
  if count == 0 {
    return Vec::new();
  }

  assert!(min <= max, "min must be less than or equal to max");

  let range_size = usize::from(max) - usize::from(min) + 1;
  assert!(
    count <= range_size,
    "count {} exceeds unique numbers available in range [{}, {}]",
    count,
    min,
    max
  );

  let mut rng = ChaCha20Rng::from_seed(seed);
  let mut numbers: Vec<u8> = (min..=max).collect();

  // 中文：仅执行前 count 轮交换，等价于部分 Fisher-Yates 洗牌，避免不必要的全量打乱。
  // EN: Only the first count swaps are performed, which is equivalent to a partial Fisher-Yates shuffle and avoids a full shuffle.
  for index in 0..count {
    let swap_index = rng.random_range(index..numbers.len());
    numbers.swap(index, swap_index);
  }

  numbers.truncate(count);
  numbers
}

impl ChaCha20RNGGenerator for u8 {
  fn chacha20_rand(seed: [u8; 32], min: Self, max: Self, count: usize) -> Vec<Self> {
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
      result.push(rng.random_range(min..=max));
    }
    result
  }
}

impl ChaCha20RNGGenerator for u16 {
  fn chacha20_rand(seed: [u8; 32], min: Self, max: Self, count: usize) -> Vec<Self> {
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
      result.push(rng.random_range(min..=max));
    }
    result
  }
}

impl ChaCha20RNGGenerator for u32 {
  fn chacha20_rand(seed: [u8; 32], min: Self, max: Self, count: usize) -> Vec<Self> {
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
      result.push(rng.random_range(min..=max));
    }
    result
  }
}

impl ChaCha20RNGGenerator for u64 {
  fn chacha20_rand(seed: [u8; 32], min: Self, max: Self, count: usize) -> Vec<Self> {
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
      result.push(rng.random_range(min..=max));
    }
    result
  }
}

/// 中文：在闭区间 [min, max] 内生成 count 个随机值，但不保证结果唯一。
/// EN: Generates count random values within the inclusive range [min, max], but does not guarantee uniqueness.
pub fn generate_numbers<T: SampleUniform + PartialOrd + Copy>(seed: [u8; 32], min: T, max: T, count: usize) -> Vec<T> {
  let mut rng = ChaCha20Rng::from_seed(seed);
  let mut result: Vec<T> = Vec::with_capacity(count);
  for _ in 0..count {
    result.push(rng.random_range(min..=max));
  }
  result
}

/// 中文：验证通用随机数生成与唯一随机数生成的边界和基本行为。
/// EN: Verifies the basic behavior and boundary conditions of the generic and unique random number generators.
#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;

  #[test]
  fn test_generate_numbers_u8() {
    let seed = [0u8; 32];
    let min = 10u8;
    let max = 20u8;
    let count = 5;
    let numbers = generate_numbers(seed, min, max, count);
    assert_eq!(numbers.len(), count);
    for &num in &numbers {
      assert!(num >= min && num <= max);
    }
  }

  #[test]
  fn test_generate_numbers_u16() {
    let seed = [0u8; 32];
    let min = 1000u16;
    let max = 2000u16;
    let count = 5;
    let numbers = generate_numbers(seed, min, max, count);
    assert_eq!(numbers.len(), count);
    for &num in &numbers {
      assert!(num >= min && num <= max);
    }
  }

  #[test]
  fn test_generate_numbers_u32() {
    let seed = [0u8; 32];
    let min = 100000u32;
    let max = 200000u32;
    let count = 5;
    let numbers = generate_numbers(seed, min, max, count);
    assert_eq!(numbers.len(), count);
    for &num in &numbers {
      assert!(num >= min && num <= max);
    }
  }

  #[test]
  fn test_generate_numbers_u64() {
    let seed = [0u8; 32];
    let min = 10000000000u64;
    let max = 20000000000u64;
    let count = 5;
    let numbers = generate_numbers(seed, min, max, count);
    assert_eq!(numbers.len(), count as usize);
    for &num in &numbers {
      assert!(num >= min && num <= max);
    }
  }

  #[test]
  fn test_generate_unique_numbers_u8() {
    let seed = [9u8; 32];
    let min = 1u8;
    let max = 40u8;
    let count = 20;
    let numbers = generate_unique_numbers(seed, min, max, count);

    assert_eq!(numbers.len(), count);
    assert_eq!(numbers.iter().copied().collect::<HashSet<_>>().len(), count);
    for &num in &numbers {
      assert!(num >= min && num <= max);
    }
  }

  #[test]
  #[should_panic(expected = "exceeds unique numbers available in range")]
  fn test_generate_unique_numbers_rejects_excessive_count() {
    let seed = [11u8; 32];
    let _ = generate_unique_numbers(seed, 1, 3, 4);
  }
}
