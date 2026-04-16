use std::cell::RefCell;
use std::collections::VecDeque;

thread_local! {
  static POOL: RefCell<VecDeque<crate::RawSeed>> = RefCell::new(VecDeque::new());
  /// 正在飞行中（已 spawn、尚未落池）的补充任务数。
  static IN_FLIGHT: RefCell<usize> = RefCell::new(0);
  /// 最近一次补充失败的时间戳（IC nanoseconds）。None 表示无失败记录。
  static LAST_FAILURE_AT: RefCell<Option<u64>> = RefCell::new(None);
}

fn target_size() -> usize {
  crate::SEED_POOL_CONFIG.with(|config| config.borrow().get().target_size as usize)
}

fn failure_cooldown_nanos() -> u64 {
  crate::SEED_POOL_CONFIG.with(|config| {
    config
      .borrow()
      .get()
      .failure_cooldown_secs
      .saturating_mul(1_000_000_000)
  })
}

fn clear_cooldown() {
  LAST_FAILURE_AT.with(|failed_at| {
    *failed_at.borrow_mut() = None;
  });
}

fn is_in_cooldown() -> bool {
  let cooldown_nanos = failure_cooldown_nanos();
  if cooldown_nanos == 0 {
    return false;
  }

  LAST_FAILURE_AT.with(|t| {
    t.borrow().map_or(false, |failed_at| {
      ic_cdk::api::time() < failed_at.saturating_add(cooldown_nanos)
    })
  })
}

/// 从池中取出一颗种子。返回 None 表示池空。
pub fn pop() -> Option<crate::RawSeed> {
  let result = POOL.with(|p| p.borrow_mut().pop_front());
  crate::debug_println!(
    "[seed_pool] pop: got={}, remaining={}",
    result.is_some(),
    pool_size()
  );
  result
}

pub fn pool_size() -> usize {
  POOL.with(|p| p.borrow().len())
}

/// 返回当前正在飞行中的补充任务数。
pub fn get_in_flight() -> usize {
  IN_FLIGHT.with(|f| *f.borrow())
}

/// 返回最近一次补充失败的时间戳（IC nanoseconds）。
pub fn get_last_failure_at() -> Option<u64> {
  LAST_FAILURE_AT.with(|t| *t.borrow())
}

pub fn apply_config_change() {
  crate::debug_println!("[seed_pool] apply_config_change: clearing cooldown and triggering refill");
  clear_cooldown();
  trigger_refill();
}

/// 检查池是否需要补充，按需 spawn 后台任务填满至 TARGET_SIZE。
/// VRF 失败后进入冷却期（FAILURE_COOLDOWN_SECS），期间跳过 spawn。
/// 可在任意同步上下文调用，不阻塞调用方。
pub fn trigger_refill() {
  let target = target_size();
  if target == 0 {
    return;
  }
  if is_in_cooldown() {
    crate::debug_println!(
      "[seed_pool] trigger_refill skipped: in cooldown, last_failure_at={:?}",
      get_last_failure_at()
    );
    return;
  }

  let available = pool_size().saturating_add(get_in_flight());
  if available >= target {
    crate::debug_println!(
      "[seed_pool] trigger_refill skipped: already sufficient, pool={}, in_flight={}, target={}",
      pool_size(),
      get_in_flight(),
      target
    );
    return;
  }

  let to_spawn = target - available;
  crate::debug_println!(
    "[seed_pool] trigger_refill: spawning {} tasks, pool={}, in_flight={}, target={}",
    to_spawn,
    pool_size(),
    get_in_flight(),
    target
  );
  IN_FLIGHT.with(|f| *f.borrow_mut() += to_spawn);

  for _ in 0..to_spawn {
    ic_cdk::futures::spawn(async {
      let result = crate::ic_rand_utils::try_get_on_chain_seed().await;

      IN_FLIGHT.with(|f| {
        let mut n = f.borrow_mut();
        *n = n.saturating_sub(1);
      });

      match result {
        Ok(seed) => {
          clear_cooldown();
          POOL.with(|p| p.borrow_mut().push_back(seed));
        }
        Err(e) => {
          LAST_FAILURE_AT.with(|t| *t.borrow_mut() = Some(ic_cdk::api::time()));
          ic_cdk::println!("[seed_pool] refill error: {}", e);
        }
      }
    });
  }
}
