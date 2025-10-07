use crate::common::chacha::ChaCha20RNGGenerator;

/// generate a random ball between min and max using ChaCha20RNGGenerator
pub fn generate_ball(seed: [u8;32], min: u8, max: u8) -> u8 {
  u8::chacha20_rand(seed, min, max, 1)[0]
}

/// test generate_ball
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_generate_ball() {
    let seed = [0u8; 32];

    let ball = generate_ball(seed, 1, 10);
    println!("Generated ball: {}", ball);
    assert!(ball >= 1 && ball <= 10);
  }
}