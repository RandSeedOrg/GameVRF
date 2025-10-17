use rand::{distr::uniform::{SampleUniform}, Rng};
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;

pub trait ChaCha20RNGGenerator {
  fn chacha20_rand(seed: [u8;32], min: Self, max: Self, count: usize) -> Vec<Self> where Self: Sized;
}

impl ChaCha20RNGGenerator for u8 {
  fn chacha20_rand(seed: [u8;32], min: Self, max: Self, count: usize) -> Vec<Self> {
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
      result.push(rng.random_range(min..=max));
    }
    result
  }
}

impl ChaCha20RNGGenerator for u16 {
  fn chacha20_rand(seed: [u8;32], min: Self, max: Self, count: usize) -> Vec<Self> {
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
      result.push(rng.random_range(min..=max));
    }
    result
  }
}

impl ChaCha20RNGGenerator for u32 {
  fn chacha20_rand(seed: [u8;32], min: Self, max: Self, count: usize) -> Vec<Self> {
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
      result.push(rng.random_range(min..=max));
    }
    result
  }
}

impl ChaCha20RNGGenerator for u64 {
  fn chacha20_rand(seed: [u8;32], min: Self, max: Self, count: usize) -> Vec<Self> {
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
      result.push(rng.random_range(min..=max));
    }
    result
  }
}


pub fn generate_numbers<T: SampleUniform + PartialOrd + Copy>(seed: [u8;32], min: T, max: T, count: usize) -> Vec<T> {
  let mut rng = ChaCha20Rng::from_seed(seed);
  let mut result:Vec<T> = Vec::with_capacity(count);
  for _ in 0..count {
    result.push(rng.random_range(min..=max));
  }
  result
}

/// test generate_numbers
#[cfg(test)]
mod tests {
  use super::*;
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
}