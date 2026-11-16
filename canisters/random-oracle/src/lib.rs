use std::{
  cell::RefCell,
  collections::{BTreeSet, HashMap, HashSet},
};

/// 仅在 sensitive_debug_api feature 开启时打印日志，用法同 ic_cdk::println!
#[macro_export]
macro_rules! debug_println {
  ($($arg:tt)*) => {
    #[cfg(feature = "sensitive_debug_api")]
    ic_cdk::println!($($arg)*);
  };
}

use candid::{CandidType, Principal};
use ic_cdk::{
  api::{accept_message, is_controller, msg_caller, msg_method_name, time},
  init, inspect_message, post_upgrade, query, update,
};
use ic_stable_structures::{
  memory_manager::{MemoryManager, VirtualMemory},
  BTreeMap as StableBTreeMap, Cell, DefaultMemoryImpl,
};
use serde::Deserialize;

#[cfg(feature = "sensitive_debug_api")]
use crate::transport_structures::SeedPoolStatus;
use crate::{
  ic_rand_utils::get_on_chain_seed,
  memory_ids::{
    AUTHORIZED_CALLERS_MEMORY_ID, AUTHORIZED_CALLERS_SETUP_STATE_MEMORY_ID, RAND_SEED_MEMORY_ID, RAND_SEED_MEMORY_SEQ_MEMORY_ID,
    SEED_POOL_CONFIG_MEMORY_ID,
  },
  stable_structures::{BusinessType, RandSeed, Scene, SeedPoolConfig},
  transport_structures::RandSeedVO,
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

const AUTHORIZED_CALLERS_NEEDS_SETUP: u8 = 1;
const AUTHORIZED_CALLERS_READY: u8 = 2;
const MAX_AUTHORIZED_CALLERS: usize = 100;
const AUTHORIZED_CALLER_MANAGEMENT_PERMISSION: &str = "random_oracle::authorized_caller::manage";

#[derive(CandidType, Deserialize)]
pub struct UserRolePermissionVo {
  pub principal_id: String,
  pub permission_codes: Vec<String>,
}

#[derive(CandidType, Deserialize)]
pub struct SystemConfig {
  pub user_role_permissions: Vec<UserRolePermissionVo>,
}

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

  static USER_ROLE_PERMISSIONS: RefCell<HashMap<String, HashSet<String>>> = RefCell::new(HashMap::new());

  pub static AUTHORIZED_CALLERS_SETUP_STATE: RefCell<Cell<u8, Memory>> = RefCell::new(
    Cell::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(AUTHORIZED_CALLERS_SETUP_STATE_MEMORY_ID)),
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

fn has_permission(caller: &Principal, permission_code: &str) -> bool {
  USER_ROLE_PERMISSIONS.with(|permissions| {
    permissions
      .borrow()
      .get(&caller.to_text())
      .is_some_and(|codes| codes.contains(permission_code))
  })
}

fn check_authorized_caller_management_authorization(caller: Principal, permission_granted: bool) -> Result<(), String> {
  if caller == Principal::anonymous() {
    return Err("Anonymous caller is not allowed".to_string());
  }
  if !permission_granted {
    return Err("Caller does not have permission to manage random oracle authorized callers".to_string());
  }
  Ok(())
}

fn require_authorized_caller_management() -> Result<(), String> {
  let caller = msg_caller();
  check_authorized_caller_management_authorization(caller, has_permission(&caller, AUTHORIZED_CALLER_MANAGEMENT_PERMISSION))
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

fn check_authorized_caller_capacity(current_count: u64, added_count: usize) -> Result<(), String> {
  if added_count > MAX_AUTHORIZED_CALLERS || current_count > MAX_AUTHORIZED_CALLERS as u64 - added_count as u64 {
    return Err(format!("Authorized caller limit of {} reached", MAX_AUTHORIZED_CALLERS));
  }
  Ok(())
}

fn replace_user_role_permissions(user_role_permissions: Vec<UserRolePermissionVo>) {
  let permissions = user_role_permissions
    .into_iter()
    .map(|entry| (entry.principal_id, entry.permission_codes.into_iter().collect()))
    .collect();
  USER_ROLE_PERMISSIONS.with(|current| {
    *current.borrow_mut() = permissions;
  });
}

fn clear_authorized_callers() {
  AUTHORIZED_CALLERS.with(|callers| {
    let keys: Vec<Principal> = callers.borrow().iter().map(|entry| *entry.key()).collect();
    let mut map = callers.borrow_mut();
    for key in keys {
      map.remove(&key);
    }
  });
}

/// The legacy migration cannot distinguish trusted canisters from arbitrary historical callers.
fn prepare_authorized_callers_for_upgrade() {
  let state = AUTHORIZED_CALLERS_SETUP_STATE.with(|state| *state.borrow().get());
  if state == 0 {
    clear_authorized_callers();
    AUTHORIZED_CALLERS_SETUP_STATE.with(|state| {
      state.borrow_mut().set(AUTHORIZED_CALLERS_NEEDS_SETUP);
    });
  }
}

fn validate_initial_authorized_callers(principals: Vec<Principal>) -> Result<BTreeSet<Principal>, String> {
  if principals.is_empty() {
    return Err("At least one trusted caller is required for initialization".to_string());
  }
  if principals.len() > MAX_AUTHORIZED_CALLERS {
    return Err(format!("At most {} trusted callers may be initialized", MAX_AUTHORIZED_CALLERS));
  }
  for principal in &principals {
    validate_whitelist_target(principal)?;
  }
  Ok(principals.into_iter().collect())
}

fn initialize_authorized_callers_from_config(principals: Vec<Principal>) -> Result<(), String> {
  let state = AUTHORIZED_CALLERS_SETUP_STATE.with(|state| *state.borrow().get());
  if state == AUTHORIZED_CALLERS_READY {
    return Ok(());
  }
  if state != AUTHORIZED_CALLERS_NEEDS_SETUP {
    return Err("Authorized callers are not ready for initialization".to_string());
  }

  let unique_principals = validate_initial_authorized_callers(principals)?;
  AUTHORIZED_CALLERS.with(|callers| -> Result<(), String> {
    let mut map = callers.borrow_mut();
    let added_count = unique_principals.iter().filter(|principal| !map.contains_key(principal)).count();
    check_authorized_caller_capacity(map.len(), added_count)?;
    for principal in unique_principals {
      map.insert(principal, ());
    }
    Ok(())
  })?;

  AUTHORIZED_CALLERS_SETUP_STATE.with(|state| {
    state.borrow_mut().set(AUTHORIZED_CALLERS_READY);
  });
  Ok(())
}

#[init]
fn init() {
  AUTHORIZED_CALLERS_SETUP_STATE.with(|state| {
    state.borrow_mut().set(AUTHORIZED_CALLERS_NEEDS_SETUP);
  });
}

#[post_upgrade]
fn post_upgrade() {
  prepare_authorized_callers_for_upgrade();
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
    | "initialize_authorized_callers"
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
async fn setup_subscribe(admin: Principal) -> String {
  if let Err(error) = require_controller() {
    return format!("Setup subscribe failed: {error}");
  }

  match ic_cdk::call::Call::unbounded_wait(admin, "subscribe").await {
    Ok(response) => match response.candid::<String>() {
      Ok(message) => message,
      Err(error) => format!("Failed to decode subscription response: {:?}", error),
    },
    Err(error) => format!("Failed to subscribe {:?}", error),
  }
}

#[update]
fn update_system_configs(config: SystemConfig) {
  if let Err(error) = require_controller() {
    ic_cdk::trap(&format!("Update system configs failed: {error}"));
  }

  replace_user_role_permissions(config.user_role_permissions);
}

#[update]
fn initialize_authorized_callers(principals: Vec<Principal>) -> Result<(), String> {
  require_controller()?;
  initialize_authorized_callers_from_config(principals)
}

#[update]
fn add_authorized_caller(caller: Principal) -> Result<(), String> {
  require_authorized_caller_management()?;
  validate_whitelist_target(&caller)?;
  AUTHORIZED_CALLERS.with(|callers| {
    let mut callers = callers.borrow_mut();
    if !callers.contains_key(&caller) {
      check_authorized_caller_capacity(callers.len(), 1)?;
      callers.insert(caller, ());
    }
    Ok(())
  })
}

#[update]
fn add_authorized_callers(principals: Vec<Principal>) -> Result<(), String> {
  require_authorized_caller_management()?;
  if principals.len() > MAX_AUTHORIZED_CALLERS {
    return Err(format!("A batch can contain at most {} authorized callers", MAX_AUTHORIZED_CALLERS));
  }
  for principal in &principals {
    validate_whitelist_target(principal)?;
  }

  let unique_principals: BTreeSet<Principal> = principals.into_iter().collect();
  AUTHORIZED_CALLERS.with(|callers| {
    let mut map = callers.borrow_mut();
    let added_count = unique_principals.iter().filter(|principal| !map.contains_key(principal)).count();
    check_authorized_caller_capacity(map.len(), added_count)?;
    for principal in unique_principals {
      map.insert(principal, ());
    }
    Ok(())
  })
}

#[update]
fn remove_authorized_caller(caller: Principal) -> Result<(), String> {
  require_authorized_caller_management()?;
  AUTHORIZED_CALLERS.with(|callers| {
    callers.borrow_mut().remove(&caller);
  });
  Ok(())
}

#[query(guard = "require_authorized_caller_management")]
fn get_authorized_callers() -> Vec<Principal> {
  AUTHORIZED_CALLERS.with(|callers| callers.borrow().iter().take(MAX_AUTHORIZED_CALLERS).map(|entry| *entry.key()).collect())
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
    clear_authorized_callers();
    USER_ROLE_PERMISSIONS.with(|permissions| {
      permissions.borrow_mut().clear();
    });
    AUTHORIZED_CALLERS_SETUP_STATE.with(|s| {
      s.borrow_mut().set(0);
    });
  }

  #[test]
  fn authorization_rejects_anonymous_even_if_marked() {
    assert!(check_caller_authorization(Principal::anonymous(), true, true).is_err());
    assert!(check_controller_authorization(Principal::anonymous(), true).is_err());
    assert!(check_controller_authorization(sample_principal(8), false).is_err());
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
  fn authorized_caller_management_requires_the_wlmc_permission() {
    reset_test_state();
    let manager = sample_principal(8);
    replace_user_role_permissions(vec![UserRolePermissionVo {
      principal_id: manager.to_text(),
      permission_codes: vec![AUTHORIZED_CALLER_MANAGEMENT_PERMISSION.to_string()],
    }]);

    assert!(check_authorized_caller_management_authorization(manager, has_permission(&manager, AUTHORIZED_CALLER_MANAGEMENT_PERMISSION),).is_ok());
    assert!(check_authorized_caller_management_authorization(
      sample_principal(7),
      has_permission(&sample_principal(7), AUTHORIZED_CALLER_MANAGEMENT_PERMISSION),
    )
    .is_err());
    assert!(check_authorized_caller_management_authorization(Principal::anonymous(), true,).is_err());
    reset_test_state();
  }

  #[test]
  fn wlmc_system_config_accepts_admin_payload_with_additional_fields() {
    #[derive(CandidType)]
    struct AdminUserRolePermission {
      principal_id: String,
      is_controller: bool,
      role_codes: Vec<String>,
      permission_codes: Vec<String>,
    }

    #[derive(CandidType)]
    struct AdminSystemConfig {
      user_role_permissions: Vec<AdminUserRolePermission>,
      dicts: Vec<String>,
    }

    let payload = AdminSystemConfig {
      user_role_permissions: vec![AdminUserRolePermission {
        principal_id: sample_principal(8).to_text(),
        is_controller: false,
        role_codes: vec!["random_oracle::authorized_caller::manager".to_string()],
        permission_codes: vec![AUTHORIZED_CALLER_MANAGEMENT_PERMISSION.to_string()],
      }],
      dicts: vec!["additional admin configuration".to_string()],
    };
    let encoded = candid::encode_args((payload,)).unwrap();
    let (decoded,) = candid::decode_args::<(SystemConfig,)>(&encoded).unwrap();

    assert_eq!(
      decoded.user_role_permissions[0].permission_codes,
      vec![AUTHORIZED_CALLER_MANAGEMENT_PERMISSION.to_string()]
    );
  }

  #[test]
  fn authorized_caller_count_is_bounded_and_batch_capacity_is_checked() {
    assert!(check_authorized_caller_capacity((MAX_AUTHORIZED_CALLERS - 1) as u64, 1,).is_ok());
    assert!(check_authorized_caller_capacity(MAX_AUTHORIZED_CALLERS as u64, 1).is_err());
    assert!(check_authorized_caller_capacity(0, MAX_AUTHORIZED_CALLERS + 1).is_err());
  }

  #[test]
  fn authorized_caller_initialization_rejects_empty_anonymous_and_oversized_input() {
    reset_test_state();
    AUTHORIZED_CALLERS_SETUP_STATE.with(|state| {
      state.borrow_mut().set(AUTHORIZED_CALLERS_NEEDS_SETUP);
    });

    assert!(initialize_authorized_callers_from_config(Vec::new()).is_err());
    assert!(initialize_authorized_callers_from_config(vec![Principal::anonymous()]).is_err());
    let oversized_callers = (0..=MAX_AUTHORIZED_CALLERS).map(|id| sample_principal(id as u8)).collect();
    assert!(initialize_authorized_callers_from_config(oversized_callers).is_err());
    assert_eq!(AUTHORIZED_CALLERS.with(|callers| callers.borrow().len()), 0);
    reset_test_state();
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
  fn upgrade_discards_unverified_callers_and_initialization_is_explicit_and_idempotent() {
    reset_test_state();
    let unverified_caller = sample_principal(1);
    let trusted_caller = sample_principal(2);
    let ignored_caller = sample_principal(3);
    AUTHORIZED_CALLERS.with(|callers| {
      callers.borrow_mut().insert(unverified_caller, ());
    });

    prepare_authorized_callers_for_upgrade();
    assert!(!is_whitelisted(&unverified_caller));
    assert_eq!(AUTHORIZED_CALLERS_SETUP_STATE.with(|state| *state.borrow().get()), AUTHORIZED_CALLERS_NEEDS_SETUP);

    initialize_authorized_callers_from_config(vec![trusted_caller]).unwrap();
    initialize_authorized_callers_from_config(vec![ignored_caller]).unwrap();
    assert!(is_whitelisted(&trusted_caller));
    assert!(!is_whitelisted(&unverified_caller));
    assert!(!is_whitelisted(&ignored_caller));
    prepare_authorized_callers_for_upgrade();
    assert!(is_whitelisted(&trusted_caller));
  }
}
