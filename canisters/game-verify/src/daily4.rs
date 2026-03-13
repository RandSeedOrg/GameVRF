use ic_cdk::query;

pub type HexString = String;


/// Generate Daily4 balls based on four hex-encoded seeds
#[query]
fn generate_daily4_ball(seed1: [u8; 32], min: u8, max: u8) -> u8 {
  game_algorithms::daily4::generate_ball(seed1, min, max)
}

/// Generate Daily4 balls based on four byte-array seeds
#[query]
fn generate_daily4_balls(seed1: [u8; 32], seed2: [u8; 32], seed3: [u8; 32], seed4: [u8; 32], min: u8, max: u8) -> Vec<u8> {
  game_algorithms::daily4::generate_daily4_balls(seed1, seed2, seed3, seed4, min, max)
}
