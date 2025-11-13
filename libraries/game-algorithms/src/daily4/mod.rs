use crate::common::chacha::ChaCha20RNGGenerator;

/// generate a random ball between min and max using ChaCha20RNGGenerator
pub fn generate_ball(seed: [u8; 32], min: u8, max: u8) -> u8 {
  u8::chacha20_rand(seed, min, max, 1)[0]
}

pub fn generate_daily4_balls(seed1: [u8; 32], seed2: [u8; 32], seed3: [u8; 32], seed4: [u8; 32], min: u8, max: u8) -> Vec<u8> {
  vec![
    generate_ball(seed1, min, max),
    generate_ball(seed2, min, max),
    generate_ball(seed3, min, max),
    generate_ball(seed4, min, max),
  ]
}

/// test generate_ball
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_generate_ball() {
    let seed = [0u8; 32];

    let ball = generate_ball(seed, 0, 9);
    println!("Generated ball: {}", ball);
    assert!(ball <= 9);
  }
}