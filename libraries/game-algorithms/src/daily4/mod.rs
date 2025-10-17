use crate::common::chacha::ChaCha20RNGGenerator;

/// generate a random ball between min and max using ChaCha20RNGGenerator
pub fn generate_ball(seed: [u8; 32]) -> u8 {
  u8::chacha20_rand(seed, 0, 9, 1)[0]
}


pub fn generate_daily4_balls(seed1: [u8; 32], seed2: [u8; 32], seed3: [u8; 32], seed4: [u8; 32]) -> Vec<u8> {
  vec![
    generate_ball(seed1),
    generate_ball(seed2),
    generate_ball(seed3),
    generate_ball(seed4),
  ]
}

/// test generate_ball
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_generate_ball() {
    let seed = [0u8; 32];

    let ball = generate_ball(seed);
    println!("Generated ball: {}", ball);
    assert!(ball <= 9);
  }
}