use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::{stable_structures::{BusinessType, RandSeed, Scene}, TimestampNano};


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RandSeedVO {
  pub idx: u64,
  pub seed: [u8; 32],
  pub public_time: TimestampNano,
  pub create_time: TimestampNano,
  pub created_by: String,
  pub use_for: BusinessType,
  pub scene: Scene,
}

impl From<RandSeed> for RandSeedVO {
  fn from(seed: RandSeed) -> Self {
    RandSeedVO {
      idx: seed.get_idx(),
      seed: seed.get_seed(),
      public_time: seed.get_public_time(),
      create_time: seed.get_create_time(),
      created_by: seed.get_created_by(),
      use_for: seed.get_use_for(),
      scene: seed.get_scene(),
    }
  }
}
