use ic_cdk::management_canister::raw_rand;

// Get a random seed from ic
pub async fn get_on_chain_seed() -> [u8; 32] {
  let on_chain_seed = match raw_rand().await {
    Ok(res) => res,
    Err(err) => ic_cdk::trap(&format!("failed to get seed: {err}")),
  };

  on_chain_seed.as_slice().try_into().unwrap_or_else(|_| {
    ic_cdk::trap(&format!(
      "when creating seed from raw_rand output, expected raw randomness to be of length 32, got {}",
      on_chain_seed.len()
    ));
  })
}