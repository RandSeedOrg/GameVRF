use std::cell::RefCell;

/// 仅在 sensitive_debug_api feature 开启时打印日志，用法同 ic_cdk::println!
#[macro_export]
macro_rules! debug_println {
  ($($arg:tt)*) => {
    #[cfg(feature = "sensitive_debug_api")]
    ic_cdk::println!($($arg)*);
  };
}

use candid::Principal;
use ic_cdk::{
  api::{accept_message, is_controller, msg_caller, msg_method_name, time},
  init, inspect_message, post_upgrade, query, update,
};
use ic_stable_structures::{
  memory_manager::{MemoryManager, VirtualMemory},
  BTreeMap as StableBTreeMap, Cell, DefaultMemoryImpl,
};

use crate::{
  ic_rand_utils::get_on_chain_seed,
  memory_ids::{
    AUTHORIZED_CALLERS_MEMORY_ID, AUTH_MIGRATION_STATE_MEMORY_ID, RAND_SEED_MEMORY_ID,
    RAND_SEED_MEMORY_SEQ_MEMORY_ID, SEED_POOL_CONFIG_MEMORY_ID,
  },
  stable_structures::{BusinessType, RandSeed, Scene, SeedPoolConfig},
  transport_structures::RandSeedVO,
};
#[cfg(feature = "sensitive_debug_api")]
use crate::transport_structures::SeedPoolStatus;

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

const AUTH_MIGRATION_DONE: u8 = 1;
const MIGRATION_HEAD_SCAN_LIMIT: usize = 500;
const MIGRATION_TAIL_SCAN_LIMIT: usize = 2_000;

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

  pub static AUTHORIZED_CALLERS: RefCell<StableBTreeMap<Principal, (), Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(AUTHORIZED_CALLERS_MEMORY_ID)),
    )
  );

  pub static AUTH_MIGRATION_STATE: RefCell<Cell<u8, Memory>> = RefCell::new(
    Cell::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(AUTH_MIGRATION_STATE_MEMORY_ID)),
      0_u8,
    )
  );
}

fn check_caller_authorization(
  caller: Principal,
  caller_is_controller: bool,
  caller_is_whitelisted: bool,
) -> Result<(), String> {
  if caller == Principal::anonymous() {
    return Err("Anonymous caller is not allowed".to_string());
  }
  if caller_is_controller || caller_is_whitelisted {
    return Ok(());
  }
  Err(format!(
    "Caller {} is not authorized to generate random seeds",
    caller
  ))
}

fn check_controller_authorization(caller: Principal, caller_is_controller: bool) -> Result<(), String> {
  if caller == Principal::anonymous() {
    return Err("Anonymous caller is not allowed".to_string());
  }
  if !caller_is_controller {
    return Err("Only controllers can perform this operation".to_string());
  }
  Ok(())
}

fn is_whitelisted(caller: &Principal) -> bool {
  AUTHORIZED_CALLERS.with(|callers| callers.borrow().contains_key(caller))
}

fn require_authorized_caller() -> Result<(), String> {
  let caller = msg_caller();
  check_caller_authorization(caller, is_controller(&caller), is_whitelisted(&caller))
}

fn require_controller() -> Result<(), String> {
  let caller = msg_caller();
  check_controller_authorization(caller, is_controller(&caller))
}

fn validate_whitelist_target(principal: &Principal) -> Result<(), String> {
  if *principal == Principal::anonymous() {
    return Err("Anonymous principal cannot be added to authorized callers".to_string());
  }
  Ok(())
}

/// 首次升级到带白名单版本时，按有界窗口扫描历史种子记录中的 `created_by` Principal，
/// 自动将线上已有彩票业务 Canister 纳入授权白名单，避免大表全量遍历触发指令上限 Trap。
fn migrate_historical_callers_once() {
  let already_migrated = AUTH_MIGRATION_STATE.with(|state| *state.borrow().get() == AUTH_MIGRATION_DONE);
  if already_migrated {
    return;
  }

  let mut discovered = Vec::new();
  RAND_SEED_MAP.with(|seeds| {
    let map = seeds.borrow();
    for entry in map.iter().take(MIGRATION_HEAD_SCAN_LIMIT) {
      if let Some(principal) = entry.value().created_by {
        if principal != Principal::anonymous() && !discovered.contains(&principal) {
          discovered.push(principal);
        }
      }
    }
    for entry in map.iter().rev().take(MIGRATION_TAIL_SCAN_LIMIT) {
      if let Some(principal) = entry.value().created_by {
        if principal != Principal::anonymous() && !discovered.contains(&principal) {
          discovered.push(principal);
        }
      }
    }
  });

  AUTHORIZED_CALLERS.with(|callers| {
    let mut map = callers.borrow_mut();
    for principal in discovered {
      map.insert(principal, ());
    }
  });

  AUTH_MIGRATION_STATE.with(|state| {
    state.borrow_mut().set(AUTH_MIGRATION_DONE);
  });
}

#[init]
fn init() {
  AUTH_MIGRATION_STATE.with(|state| {
    state.borrow_mut().set(AUTH_MIGRATION_DONE);
  });
}

#[post_upgrade]
fn post_upgrade() {
  migrate_historical_callers_once();
}

/// 仅用于节省匿名垃圾请求解码 Cycles；真实安全边界由各方法内共识级校验执行。
#[inspect_message]
fn inspect_message() {
  let method = msg_method_name();
  match method.as_str() {
    "generate_rand_seed"
    | "public_rand_seed"
    | "public_rand_seeds"
    | "update_seed_pool_config"
    | "add_authorized_caller"
    | "add_authorized_callers"
    | "remove_authorized_caller" => {
      if msg_caller() != Principal::anonymous() {
        accept_message();
      }
    }
    _ => accept_message(),
  }
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

#[update(guard = "require_authorized_caller")]
async fn generate_rand_seed(use_for: BusinessType, scene: Scene) -> RandSeedVO {
  let seed = acquire_seed_from_pool().await;
  store_rand_seed(seed, use_for, scene)
}

#[update]
fn add_authorized_caller(caller: Principal) -> Result<(), String> {
  require_controller()?;
  validate_whitelist_target(&caller)?;
  AUTHORIZED_CALLERS.with(|callers| {
    callers.borrow_mut().insert(caller, ());
  });
  Ok(())
}

#[update]
fn add_authorized_callers(principals: Vec<Principal>) -> Result<(), String> {
  require_controller()?;
  for principal in &principals {
    validate_whitelist_target(principal)?;
  }
  AUTHORIZED_CALLERS.with(|callers| {
    let mut map = callers.borrow_mut();
    for principal in principals {
      map.insert(principal, ());
    }
  });
  Ok(())
}

#[update]
fn remove_authorized_caller(caller: Principal) -> Result<(), String> {
  require_controller()?;
  AUTHORIZED_CALLERS.with(|callers| {
    callers.borrow_mut().remove(&caller);
  });
  Ok(())
}

#[query(guard = "require_controller")]
fn get_authorized_callers() -> Vec<Principal> {
  AUTHORIZED_CALLERS.with(|callers| {
    callers
      .borrow()
      .iter()
      .map(|entry| *entry.key())
      .collect()
  })
}

#[query]
fn get_seed_pool_config() -> SeedPoolConfig {
  SEED_POOL_CONFIG.with(|config| *config.borrow().get())
}

/// 查询 seed pool 三个运行时存储的当前状态：
/// - pool_size: POOL 中已就绪的种子数量
/// - in_flight: IN_FLIGHT 正在异步获取中的任务数
/// - last_failure_at: LAST_FAILURE_AT 最近一次补充失败的时间戳（nanoseconds），None 表示无失败记录
///
/// 仅在 sensitive_debug_api feature 开启时可用
#[cfg(feature = "sensitive_debug_api")]
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
  let caller = msg_caller();
  if caller == Principal::anonymous() {
    return Err("Anonymous caller is not allowed".to_string());
  }
  let caller_text = caller.to_text();

  RAND_SEED_MAP.with(|seeds| {
    let mut seeds = seeds.borrow_mut();

    if let Some(mut seed) = seeds.get(&index) {
      if seed.get_created_by() != caller_text {
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
  let caller = msg_caller();
  if caller == Principal::anonymous() {
    return Err("Anonymous caller is not allowed".to_string());
  }
  let caller_text = caller.to_text();

  RAND_SEED_MAP.with(|seeds| {
    let mut seeds = seeds.borrow_mut();
    for index in indexes {
      if let Some(mut seed) = seeds.get(&index) {
        if seed.get_created_by() != caller_text {
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
    let seed = seeds.borrow().get(&index)?;
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
    indexes
      .into_iter()
      .map(|index| {
        let seed = seeds.get(&index)?;
        if seed.public_time.is_none() {
          return None;
        }
        Some(seed.into())
      })
      .collect()
  })
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
  use super::*;
  use ic_stable_structures::Storable;

  fn sample_principal(id: u8) -> Principal {
    Principal::from_slice(&[id; 29])
  }

  fn reset_test_state() {
    RAND_SEED_MAP.with(|m| {
      let keys: Vec<u64> = m.borrow().iter().map(|e| *e.key()).collect();
      let mut map = m.borrow_mut();
      for k in keys {
        map.remove(&k);
      }
    });
    AUTHORIZED_CALLERS.with(|m| {
      let keys: Vec<Principal> = m.borrow().iter().map(|e| *e.key()).collect();
      let mut map = m.borrow_mut();
      for k in keys {
        map.remove(&k);
      }
    });
    AUTH_MIGRATION_STATE.with(|s| {
      s.borrow_mut().set(0);
    });
  }

  #[test]
  fn authorization_rejects_anonymous_even_if_marked() {
    assert!(check_caller_authorization(Principal::anonymous(), true, true).is_err());
    assert!(check_controller_authorization(Principal::anonymous(), true).is_err());
    assert!(validate_whitelist_target(&Principal::anonymous()).is_err());
  }

  #[test]
  fn authorization_allows_controller_or_whitelisted_and_rejects_others() {
    let caller = sample_principal(7);
    assert!(check_caller_authorization(caller, false, false).is_err());
    assert!(check_caller_authorization(caller, true, false).is_ok());
    assert!(check_caller_authorization(caller, false, true).is_ok());
  }

  #[test]
  fn game_creator_match_seed_candid_storable_roundtrip() {
    let caller = sample_principal(9);
    let record = RandSeed {
      idx: Some(42),
      seed: Some([0xAB; 32]),
      public_time: None,
      create_time: Some(123456789),
      created_by: Some(caller),
      use_for: Some(BusinessType::GameCreator),
      scene: Some(Scene::MatchSeed),
    };
    let encoded = record.to_bytes();
    let decoded = RandSeed::from_bytes(encoded);
    assert_eq!(decoded.get_idx(), 42);
    assert_eq!(decoded.get_seed(), [0xAB; 32]);
    assert_eq!(decoded.get_use_for(), BusinessType::GameCreator);
    assert_eq!(decoded.get_scene(), Scene::MatchSeed);
    assert_eq!(decoded.get_created_by(), caller.to_text());
  }

  #[test]
  fn historical_caller_migration_populates_whitelist_once_and_ignores_anonymous() {
    reset_test_state();
    let keno_canister = sample_principal(1);
    let daily4_canister = sample_principal(2);
    let later_unmigrated = sample_principal(3);

    RAND_SEED_MAP.with(|m| {
      let mut map = m.borrow_mut();
      map.insert(
        1,
        RandSeed {
          idx: Some(1),
          seed: Some([1u8; 32]),
          public_time: None,
          create_time: Some(100),
          created_by: Some(keno_canister),
          use_for: Some(BusinessType::Keno),
          scene: Some(Scene::DrawNumbers),
        },
      );
      map.insert(
        2,
        RandSeed {
          idx: Some(2),
          seed: Some([2u8; 32]),
          public_time: None,
          create_time: Some(200),
          created_by: Some(Principal::anonymous()),
          use_for: Some(BusinessType::Daily4),
          scene: Some(Scene::DrawNumbers),
        },
      );
      map.insert(
        3,
        RandSeed {
          idx: Some(3),
          seed: Some([3u8; 32]),
          public_time: None,
          create_time: Some(300),
          created_by: Some(daily4_canister),
          use_for: Some(BusinessType::Daily4),
          scene: Some(Scene::DrawNumbers),
        },
      );
    });

    migrate_historical_callers_once();
    assert!(is_whitelisted(&keno_canister));
    assert!(is_whitelisted(&daily4_canister));
    assert!(!is_whitelisted(&Principal::anonymous()));

    // Subsequent calls must not re-run migration after AUTH_MIGRATION_DONE is set.
    RAND_SEED_MAP.with(|m| {
      m.borrow_mut().insert(
        4,
        RandSeed {
          idx: Some(4),
          seed: Some([4u8; 32]),
          public_time: None,
          create_time: Some(400),
          created_by: Some(later_unmigrated),
          use_for: Some(BusinessType::Roulette),
          scene: Some(Scene::DrawNumbers),
        },
      );
    });
    migrate_historical_callers_once();
    assert!(!is_whitelisted(&later_unmigrated));
  }
}
