use ic_cdk::query;

#[derive(candid::CandidType, candid::Deserialize)]
pub struct GenerateKenoNumbersRequest {
  pub seed: [u8; 32],
  pub round: u32,
  pub count: u32,
  pub min: Option<u8>,
  pub max: Option<u8>,
}

/// Generate Keno numbers based on a seed and round
#[query]
fn generate_keno_numbers(request: GenerateKenoNumbersRequest) -> Vec<u8> {
  let min = request.min.unwrap_or(1);
  let max = request.max.unwrap_or(40);
  game_algorithms::keno::generate_numbers(request.seed, request.round, request.count as usize, min, max)
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct GenerateKenoClientRandomNumbersRequest {
  pub seed: [u8; 32],
  pub client_timestamp: u64,
  pub count: usize,
  pub min: Option<u8>,
  pub max: Option<u8>,
}

/// Generate Keno numbers based on a seed and client timestamp
#[query]
fn generate_keno_client_random_numbers(request: GenerateKenoClientRandomNumbersRequest) -> Vec<u8> {
  let min = request.min.unwrap_or(1);
  let max = request.max.unwrap_or(40);
  game_algorithms::keno::generate_numbers(request.seed, request.client_timestamp, request.count, min, max)
}