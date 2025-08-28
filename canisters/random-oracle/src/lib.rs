use std::cell::RefCell;

use ic_cdk::{update, query, api::{time, msg_caller}};
use ic_stable_structures::{memory_manager::{MemoryManager, VirtualMemory}, BTreeMap as StableBTreeMap, Cell, DefaultMemoryImpl};

use crate::{ic_rand_utils::get_on_chain_seed, memory_ids::{RAND_SEED_MEMORY_ID, RAND_SEED_MEMORY_SEQ_MEMORY_ID}, stable_structures::{BusinessType, RandSeed, Scene}, transport_structures::RandSeedVO};

mod memory_ids;
mod ic_rand_utils;

mod stable_structures;
mod transport_structures;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type TimestampNano = u64;
type RandSeedId = u64;
pub type SeedIdGenerator = RefCell<Cell<RandSeedId, Memory>>;

thread_local! {
  static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

  pub static RAND_SEED_ID: RefCell<Cell<RandSeedId, Memory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with(|m| m.borrow().get(RAND_SEED_MEMORY_SEQ_MEMORY_ID)), 0_u64));

  pub static RAND_SEED_MAP: RefCell<StableBTreeMap<RandSeedId, RandSeed, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(RAND_SEED_MEMORY_ID)),
    )
  );
}

pub fn new_seed_id(id_seq: &SeedIdGenerator) -> RandSeedId {
  let id = id_seq.borrow().get() + 1;

  id_seq.borrow_mut().set(id);

  id
}


#[update]
async fn generate_rand_seed(use_for: BusinessType, scene: Scene) -> RandSeedVO {
  let on_chain_seed = get_on_chain_seed().await;

  let seed_id = RAND_SEED_ID.with(|id_gen| new_seed_id(id_gen));

  RAND_SEED_MAP.with(|seeds| {
    let mut seed_map = seeds.borrow_mut();

    let seed = RandSeed {
      idx:Some(seed_id),
      seed:Some(on_chain_seed),
      public_time:None,
      create_time:Some(time()),
      created_by:Some(msg_caller()),
      use_for: Some(use_for),
      scene: Some(scene),
    };

    seed_map.insert(seed_id, seed.clone());

    seed.into()
  })
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


ic_cdk::export_candid!();
