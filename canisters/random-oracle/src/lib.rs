use std::cell::RefCell;

/// 仅在 sensitive_debug_api feature 开启时打印日志，用法同 ic_cdk::println!
#[macro_export]
macro_rules! debug_println {
  ($($arg:tt)*) => {
    #[cfg(feature = "sensitive_debug_api")]
    ic_cdk::println!($($arg)*);
  };
}

use ic_cdk::{
  api::{is_controller, msg_caller, time},
  query, update,
};
use ic_stable_structures::{
  BTreeMap as StableBTreeMap, Cell, DefaultMemoryImpl,
  memory_manager::{MemoryManager, VirtualMemory},
};

use crate::{
  ic_rand_utils::get_on_chain_seed,
  memory_ids::{RAND_SEED_MEMORY_ID, RAND_SEED_MEMORY_SEQ_MEMORY_ID, SEED_POOL_CONFIG_MEMORY_ID},
  stable_structures::{BusinessType, RandSeed, Scene, SeedPoolConfig},
  transport_structures::{RandSeedVO, SeedPoolStatus},
};

mod ic_rand_utils;
mod memory_ids;
mod seed_pool;
mod stable_structures;
mod transport_structures;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type TimestampNano = u64;
type RandSeedId = u64;
pub type RawSeed = [u8; 32];
pub type SeedIdGenerator = RefCell<Cell<RandSeedId, Memory>>;

thread_local! {
  static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

  pub static RAND_SEED_ID: RefCell<Cell<RandSeedId, Memory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with(|m| m.borrow().get(RAND_SEED_MEMORY_SEQ_MEMORY_ID)), 0_u64));

  pub static RAND_SEED_MAP: RefCell<StableBTreeMap<RandSeedId, RandSeed, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(RAND_SEED_MEMORY_ID)),
    )
  );

  pub static SEED_POOL_CONFIG: RefCell<Cell<SeedPoolConfig, Memory>> = RefCell::new(
    Cell::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(SEED_POOL_CONFIG_MEMORY_ID)),
      SeedPoolConfig::default(),
    )
  );
}

pub fn new_seed_id(id_seq: &SeedIdGenerator) -> RandSeedId {
  let id = id_seq.borrow().get() + 1;

  id_seq.borrow_mut().set(id);

  id
}

fn store_rand_seed(seed_bytes: RawSeed, use_for: BusinessType, scene: Scene) -> RandSeedVO {
  let seed_id = RAND_SEED_ID.with(|id_gen| new_seed_id(id_gen));

  RAND_SEED_MAP.with(|seeds| {
    let mut seed_map = seeds.borrow_mut();

    let seed = RandSeed {
      idx: Some(seed_id),
      seed: Some(seed_bytes),
      public_time: None,
      create_time: Some(time()),
      created_by: Some(msg_caller()),
      use_for: Some(use_for),
      scene: Some(scene),
    };

    seed_map.insert(seed_id, seed.clone());

    seed.into()
  })
}

async fn acquire_seed_from_pool() -> RawSeed {
  if let Some(seed) = seed_pool::pop() {
    debug_println!("[random_oracle] acquire_seed_from_pool: hit pool, pool_size={}", seed_pool::pool_size());
    seed_pool::trigger_refill();
    return seed;
  }

  debug_println!("[random_oracle] acquire_seed_from_pool: pool empty, falling back to on-chain");
  seed_pool::trigger_refill();
  get_on_chain_seed().await
}

fn init_runtime() {
  seed_pool::apply_config_change();
}

#[ic_cdk::init]
fn init() {
  init_runtime();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
  init_runtime();
}

#[update]
async fn generate_rand_seed(use_for: BusinessType, scene: Scene) -> RandSeedVO {
  let seed = get_on_chain_seed().await;
  store_rand_seed(seed, use_for, scene)
}

#[update]
async fn generate_seed_from_pool(use_for: BusinessType, scene: Scene) -> RandSeedVO {
  let seed = acquire_seed_from_pool().await;
  store_rand_seed(seed, use_for, scene)
}

#[query]
fn get_seed_pool_config() -> SeedPoolConfig {
  SEED_POOL_CONFIG.with(|config| *config.borrow().get())
}

/// 查询 seed pool 三个运行时存储的当前状态：
/// - pool_size: POOL 中已就绪的种子数量
/// - in_flight: IN_FLIGHT 正在异步获取中的任务数
/// - last_failure_at: LAST_FAILURE_AT 最近一次补充失败的时间戳（nanoseconds），None 表示无失败记录
#[query]
fn get_seed_pool_status() -> SeedPoolStatus {
  SeedPoolStatus {
    pool_size: seed_pool::pool_size() as u64,
    in_flight: seed_pool::get_in_flight() as u64,
    last_failure_at: seed_pool::get_last_failure_at(),
  }
}

#[update]
fn update_seed_pool_config(config: SeedPoolConfig) -> Result<(), String> {
  if !is_controller(&msg_caller()) {
    return Err("Only controllers can update seed pool config".to_string());
  }

  SEED_POOL_CONFIG.with(|current| {
    current.borrow_mut().set(config);
  });
  seed_pool::apply_config_change();
  Ok(())
}

#[update]
fn public_rand_seed(index: u64) -> Result<(), String> {
  RAND_SEED_MAP.with(|seeds| {
    let mut seeds = seeds.borrow_mut();

    if let Some(mut seed) = seeds.get(&index) {
      if seed.get_created_by() != msg_caller().to_text() {
        return Err("Only the creator can publicize the seed".to_string());
      }
      seed.public_time = Some(time());
      seeds.insert(index, seed);
      Ok(())
    } else {
      Err(format!("No seed found at index {}", index))
    }
  })
}

#[update]
fn public_rand_seeds(indexes: Vec<u64>) -> Result<(), String> {
  RAND_SEED_MAP.with(|seeds| {
    let mut seeds = seeds.borrow_mut();
    for index in indexes {
      if let Some(mut seed) = seeds.get(&index) {
        if seed.get_created_by() != msg_caller().to_text() {
          return Err("Only the creator can publicize the seed".to_string());
        }
        seed.public_time = Some(time());
        seeds.insert(index, seed);
      } else {
        return Err(format!("No seed found at index {}", index));
      }
    }
    Ok(())
  })
}

#[query]
fn get_public_rand_seed(index: u64) -> Option<RandSeedVO> {
  RAND_SEED_MAP.with(|seeds| {
    let seed = seeds.borrow().get(&index);

    if seed.is_none() {
      return None;
    }

    let seed = seed.unwrap();

    if seed.public_time.is_none() {
      return None;
    }

    Some(seed.into())
  })
}

#[query]
fn get_public_rand_seeds(indexes: Vec<u64>) -> Vec<Option<RandSeedVO>> {
  RAND_SEED_MAP.with(|seeds| {
    let seeds = seeds.borrow();
    indexes.into_iter().map(|index| {
      let seed = seeds.get(&index);

      if seed.is_none() {
        return None;
      }

      let seed = seed.unwrap();

      if seed.public_time.is_none() {
        return None;
      }

      Some(seed.into())
    }).collect()
  })
}

ic_cdk::export_candid!();
