use std::cell::RefCell;

use ic_cdk::{update, query, api::{time, msg_caller}};
use ic_stable_structures::{memory_manager::{VirtualMemory, MemoryManager}, DefaultMemoryImpl, Vec as StableVec};

use crate::{ic_rand_utils::get_on_chain_seed, memory_ids::RAND_SEED_MEMORY_ID, stable_structures::{BusinessType, RandSeed, Scene}, transport_structures::RandSeedVO};

mod memory_ids;
mod ic_rand_utils;

mod stable_structures;
mod transport_structures;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type TimestampNano = u64;

thread_local! {
  static RAND_SEEDS: RefCell<StableVec<RandSeed, Memory>> = {
    let memory_manager = MemoryManager::init(DefaultMemoryImpl::default());
    RefCell::new(StableVec::new(
      memory_manager.get(RAND_SEED_MEMORY_ID),
    ))
  };
}


#[update]
async fn generate_rand_seed(use_for: BusinessType, scene: Scene) -> RandSeedVO {
  let on_chain_seed = get_on_chain_seed().await;

  RAND_SEEDS.with(|seeds| {
    let seeds = seeds.borrow_mut();

    let seed = RandSeed {
      idx:Some(seeds.len() as u64),
      seed:Some(on_chain_seed),
      public_time:None,
      create_time:Some(time()),
      created_by:Some(msg_caller()),
      use_for: Some(use_for),
      scene: Some(scene),
    };

    seeds.push(&seed);

    seed.into()
  })
}

#[update]
fn public_rand_seed(index: u64) -> Result<(), String> {
  RAND_SEEDS.with(|seeds| {
    let seeds = seeds.borrow_mut();

    if let Some(mut seed) = seeds.get(index) {
      if seed.get_created_by() != msg_caller().to_text() {
        return Err("Only the creator can publicize the seed".to_string());
      }
      seed.public_time = Some(time());
      seeds.set(index, &seed);
      Ok(())
    } else {
      Err(format!("No seed found at index {}", index))
    }
  })
}

#[query]
fn get_public_rand_seed(index: u64) -> Option<RandSeedVO> {
  RAND_SEEDS.with(|seeds| {
    let seed = seeds.borrow().get(index);

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

