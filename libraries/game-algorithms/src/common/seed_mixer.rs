use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Mixes a 32-byte root seed with an arbitrary byte slice to generate a new 32-byte seed.
/// 将 32 字节的根种子与任意字节切片混合，生成一个新的 32 字节种子。
///
/// This function uses XOR for initial mixing and then applies ChaCha20 PRNG to ensure cryptographic security and unpredictability.
/// 该函数首先使用异或（XOR）进行初始混合，然后应用 ChaCha20 伪随机数生成器以确保密码学级别的安全性和不可预测性。
pub fn seed_mix(mut seed: [u8; 32], mix_data: &[u8]) -> [u8; 32] {
  // XOR the first few bytes of the seed with the provided data slice (up to 32 bytes)
  // 将提供的字节数据（最多 32 字节）与种子进行异或操作
  for (i, &b) in mix_data.iter().take(32).enumerate() {
    seed[i] ^= b;
  }

  // Initialize ChaCha20 RNG with the modified seed
  // 使用修改后的种子初始化 ChaCha20 随机数生成器
  let mut rng = ChaCha20Rng::from_seed(seed);

  // Generate and return a new 32-byte seed
  // 生成并返回全新的 32 字节种子
  let mut new_seed = [0u8; 32];
  rng.fill_bytes(&mut new_seed);
  new_seed
}

/// A trait for numeric types that can be mixed into a 32-byte seed.
/// 用于能够混入 32 字节种子的数字类型的 Trait。
pub trait SeedMixableNumber {
  /// Mixes the numeric value into the provided 32-byte seed and returns the new seed.
  /// 将该数字值混入提供的 32 字节种子中，并返回新的种子。
  fn mix_into_seed(self, seed: [u8; 32]) -> [u8; 32];
}

/// Macro to implement the `SeedMixableNumber` trait for various numeric types.
/// 用于为多种数字类型快捷实现 `SeedMixableNumber` Trait 的宏（Macro）。
macro_rules! impl_seed_mixable_number {
  ($($t:ty),*) => {
    $(
      impl SeedMixableNumber for $t {
        fn mix_into_seed(self, seed: [u8; 32]) -> [u8; 32] {
          // Convert the number to little-endian bytes and use the underlying byte-slice mix function
          // 将数字转换为小端字节数组，并调用底层的字节切片混入函数
          seed_mix(seed, &self.to_le_bytes())
        }
      }
    )*
  };
}

// Implement the trait for all supported unsigned integer types
// 为所有受支持的无符号整数类型自动派生实现该 Trait
impl_seed_mixable_number!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize);

/// A generic public function to mix any supported numeric type into a seed.
/// 泛型的公共接口函数，支持将任意受支持的数字类型（如 u8, u32, u64 等）混入种子。
///
/// This avoids the need for callers to manually handle byte conversions.
/// 它可以避免调用方在每次调用时都手动去处理字节数组的转换操作（to_le_bytes）。
pub fn mix_number_to_seed<T: SeedMixableNumber>(seed: [u8; 32], num: T) -> [u8; 32] {
  num.mix_into_seed(seed)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_seed_mix_determinism() {
    let seed = [1u8; 32];
    let data = b"test_data";
    let mixed1 = seed_mix(seed, data);
    let mixed2 = seed_mix(seed, data);
    assert_eq!(mixed1, mixed2, "Same seed and data should produce same mixed seed");
  }

  #[test]
  fn test_seed_mix_differs() {
    let seed = [2u8; 32];
    let data1 = b"test_data_1";
    let data2 = b"test_data_2";

    let mixed1 = seed_mix(seed, data1);
    let mixed2 = seed_mix(seed, data2);
    assert_ne!(mixed1, mixed2, "Different data should produce different mixed seeds");
  }

  #[test]
  fn test_mix_number_to_seed() {
    let seed = [3u8; 32];

    // Note: because we use little-endian byte representation and XOR logic,
    // mixing `10u8` ([10]) and `10u32` ([10, 0, 0, 0]) produces the exact same intermediate seed,
    // because XORing with 0 does not change the state.
    let mixed_u8 = mix_number_to_seed(seed, 10u8);
    let mixed_u32 = mix_number_to_seed(seed, 10u32);
    let mixed_u64 = mix_number_to_seed(seed, 10u64);

    assert_eq!(mixed_u8, mixed_u32, "Different types with the same little-endian byte value yield the same result");
    assert_eq!(mixed_u32, mixed_u64);

    // However, if we use a value that utilizes higher bytes (e.g. 256), the results will differ.
    // 256u32 is [0, 1, 0, 0]. 256 cannot fit in u8, but compared to 0u8 ([0]), they will be different.
    let mixed_0u8 = mix_number_to_seed(seed, 0u8);
    let mixed_256u32 = mix_number_to_seed(seed, 256u32);
    assert_ne!(mixed_0u8, mixed_256u32, "Different numeric values must yield different seeds");

    let mixed_u32_again = mix_number_to_seed(seed, 10u32);
    assert_eq!(mixed_u32, mixed_u32_again, "Same type and value should be deterministic");
  }

  #[test]
  fn test_trait_direct_usage() {
    let seed = [4u8; 32];
    let round = 100u32;

    // Test that invoking the trait method directly works identical to the generic function
    let mixed_direct = round.mix_into_seed(seed);
    let mixed_wrapper = mix_number_to_seed(seed, round);

    assert_eq!(mixed_direct, mixed_wrapper);
  }
}
