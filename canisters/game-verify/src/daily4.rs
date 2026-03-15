use ic_cdk::query;

#[derive(candid::CandidType, candid::Deserialize)]
pub struct GenerateDaily4BallRequest {
  pub seed: [u8; 32],
  pub ball_range_min: u8,
  pub ball_range_max: u8,
}

/// Generate Daily4 balls based on four hex-encoded seeds
#[query]
fn generate_daily4_ball(request: GenerateDaily4BallRequest) -> u8 {
  game_algorithms::daily4::generate_ball(request.seed, request.ball_range_min, request.ball_range_max)
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct GenerateDaily4BallsRequest {
  pub seed1: [u8; 32],
  pub seed2: [u8; 32],
  pub seed3: [u8; 32],
  pub seed4: [u8; 32],
  pub ball_range_min: u8,
  pub ball_range_max: u8,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct GenerateDaily4BallsResponse {
  pub ball1: u8,
  pub ball2: u8,
  pub ball3: u8,
  pub ball4: u8,
}

/// Generate Daily4 balls based on four byte-array seeds
#[query]
fn generate_daily4_balls(request: GenerateDaily4BallsRequest) -> GenerateDaily4BallsResponse {
  let balls = game_algorithms::daily4::generate_daily4_balls(
    request.seed1,
    request.seed2,
    request.seed3,
    request.seed4,
    request.ball_range_min,
    request.ball_range_max,
  );
  let [ball1, ball2, ball3, ball4]: [u8; 4] = balls
    .try_into()
    .expect("Expected exactly 4 balls");
  GenerateDaily4BallsResponse { ball1, ball2, ball3, ball4 }
}
