use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{stable_structures::{BusinessType, RandSeed, Scene}, TimestampNano};

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub enum LegacyBusinessType {
  LuckyNickel,
  QuickQuid,
  RoyalTreys,
  Daily4,
  Mines,
  Keno,
  FruitsGarden,
  Roulette,
}

impl TryFrom<BusinessType> for LegacyBusinessType {
  type Error = ();

  fn try_from(value: BusinessType) -> Result<Self, Self::Error> {
    Ok(match value {
      BusinessType::LuckyNickel => Self::LuckyNickel,
      BusinessType::QuickQuid => Self::QuickQuid,
      BusinessType::RoyalTreys => Self::RoyalTreys,
      BusinessType::Daily4 => Self::Daily4,
      BusinessType::Mines => Self::Mines,
      BusinessType::Keno => Self::Keno,
      BusinessType::FruitsGarden => Self::FruitsGarden,
      BusinessType::Roulette => Self::Roulette,
      BusinessType::GameCreator => return Err(()),
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub enum LegacyScene {
  GenerateTicketPool,
  Shuffle,
  DrawNumbers,
}

impl TryFrom<Scene> for LegacyScene {
  type Error = ();

  fn try_from(value: Scene) -> Result<Self, Self::Error> {
    Ok(match value {
      Scene::GenerateTicketPool => Self::GenerateTicketPool,
      Scene::Shuffle => Self::Shuffle,
      Scene::DrawNumbers => Self::DrawNumbers,
      Scene::MatchSeed => return Err(()),
    })
  }
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct LegacyRandSeedVO {
  pub idx: u64,
  pub seed: crate::RawSeed,
  pub public_time: TimestampNano,
  pub create_time: TimestampNano,
  pub created_by: String,
  pub use_for: LegacyBusinessType,
  pub scene: LegacyScene,
}

impl TryFrom<RandSeedVO> for LegacyRandSeedVO {
  type Error = ();

  fn try_from(seed: RandSeedVO) -> Result<Self, Self::Error> {
    Ok(Self {
      idx: seed.idx,
      seed: seed.seed,
      public_time: seed.public_time,
      create_time: seed.create_time,
      created_by: seed.created_by,
      use_for: seed.use_for.try_into()?,
      scene: seed.scene.try_into()?,
    })
  }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SeedPoolStatus {
  pub pool_size: u64,
  pub in_flight: u64,
  pub last_failure_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RandSeedVO {
  pub idx: u64,
  pub seed: crate::RawSeed,
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
