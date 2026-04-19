use std::borrow::Cow;

use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Serialize, Deserialize};

use crate::TimestampNano;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum BusinessType {
  LuckyNickel,
  QuickQuid,
  RoyalTreys,
  Daily4,
  Mines,
  Keno,
  FruitsGarden,
}

impl Default for BusinessType {
  fn default() -> Self {
    BusinessType::LuckyNickel
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum Scene {
  GenerateTicketPool,
  Shuffle,
  DrawNumbers,
}

impl Default for Scene {
  fn default() -> Self {
    Scene::Shuffle
  }
}

pub const DEFAULT_SEED_POOL_TARGET_SIZE: u32 = 10;
pub const DEFAULT_SEED_POOL_FAILURE_COOLDOWN_SECS: u64 = 60;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, CandidType)]
pub struct SeedPoolConfig {
  pub target_size: u32,             // pool size
  pub failure_cooldown_secs: u64,   // unit:second
}

impl Default for SeedPoolConfig {
  fn default() -> Self {
    Self {
      target_size: DEFAULT_SEED_POOL_TARGET_SIZE,
      failure_cooldown_secs: DEFAULT_SEED_POOL_FAILURE_COOLDOWN_SECS,
    }
  }
}

impl Storable for SeedPoolConfig {
  fn to_bytes(&self) -> Cow<'_, [u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  fn into_bytes(self) -> Vec<u8> {
    Encode!(&self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RandSeed {
  pub idx: Option<u64>,
  pub seed: Option<crate::RawSeed>,
  pub public_time: Option<TimestampNano>,
  pub create_time: Option<TimestampNano>,
  pub created_by: Option<Principal>,
  pub use_for: Option<BusinessType>,
  pub scene: Option<Scene>,
}

impl RandSeed {
  pub fn get_idx(&self) -> u64 {
    self.idx.unwrap_or_default()
  }

  pub fn get_seed(&self) -> crate::RawSeed {
    self.seed.unwrap_or_default()
  }

  pub fn get_public_time(&self) -> TimestampNano {
    self.public_time.unwrap_or_default()
  }

  pub fn get_create_time(&self) -> TimestampNano {
    self.create_time.unwrap_or_default()
  }

  pub fn get_created_by(&self) -> String {
    self.created_by.map(|p| p.to_text()).unwrap_or_default()
  }

  pub fn get_use_for(&self) -> BusinessType {
    self.use_for.clone().unwrap_or_default()
  }

  pub fn get_scene(&self) -> Scene {
    self.scene.clone().unwrap_or_default()
  }
}

impl Storable for RandSeed {
  fn to_bytes(&self) -> Cow<'_, [u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  fn into_bytes(self) -> Vec<u8> {
    Encode!(&self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
