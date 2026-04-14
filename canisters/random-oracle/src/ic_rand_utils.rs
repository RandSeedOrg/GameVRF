use ic_cdk::management_canister::raw_rand;

async fn get_on_chain_seed_result() -> Result<crate::RawSeed, String> {
  let on_chain_seed = raw_rand()
    .await
    .map_err(|err| format!("failed to get seed: {err}"))?;

  on_chain_seed.as_slice().try_into().map_err(|_| {
    format!(
      "when creating seed from raw_rand output, expected raw randomness to be of length 32, got {}",
      on_chain_seed.len()
    )
  })
}

pub async fn try_get_on_chain_seed() -> Result<crate::RawSeed, String> {
  // seed_pool 的后台 refill 不能 trap，否则无法记录失败时间并进入 cooldown。
  get_on_chain_seed_result().await
}

// Get a random seed from ic
pub async fn get_on_chain_seed() -> crate::RawSeed {
  get_on_chain_seed_result()
    .await
    .unwrap_or_else(|err| ic_cdk::trap(&err))
}
