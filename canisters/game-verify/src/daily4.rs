use game_algorithms::{common::seed_format::hex_to_seed};
use ic_cdk::query;

pub type HexString = String;


/// Generate Daily4 balls based on four hex-encoded seeds
#[query]
fn generate_daily4_ball_by_hex_seeds(seed1: HexString, seed2: HexString, seed3: HexString, seed4: HexString, min: u8, max: u8) -> Result<Vec<u8>, String> {
  let seed1 = hex_to_seed(&seed1).map_err(|e| format!("Failed to decode seed1: {}", e))?;
  let seed2 = hex_to_seed(&seed2).map_err(|e| format!("Failed to decode seed2: {}", e))?;
  let seed3 = hex_to_seed(&seed3).map_err(|e| format!("Failed to decode seed3: {}", e))?;
  let seed4 = hex_to_seed(&seed4).map_err(|e| format!("Failed to decode seed4: {}", e))?;
  Ok(game_algorithms::daily4::generate_daily4_balls(seed1, seed2, seed3, seed4, min, max))
}

/// Generate Daily4 balls based on four byte-array seeds
#[query]
fn generate_daily4_balls(seed1: [u8; 32], seed2: [u8; 32], seed3: [u8; 32], seed4: [u8; 32], min: u8, max: u8) -> Vec<u8> {
  game_algorithms::daily4::generate_daily4_balls(seed1, seed2, seed3, seed4, min, max)
}