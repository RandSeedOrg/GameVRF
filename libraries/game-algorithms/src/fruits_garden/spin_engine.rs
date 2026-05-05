use std::collections::HashSet;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

use super::types::{
  AlgorithmPayoutRule, CascadeResolution, SpinAlgorithmConfig, SpinEngineInput, SpinEngineOutput, SymbolCode, WinningLineResolution,
};

#[derive(Clone, Debug, Default)]
struct HeldSpinContext {
  held_symbols: HashSet<SymbolCode>,
  held_count: u8,
}

pub fn execute_spin(input: SpinEngineInput) -> SpinEngineOutput {
  let layout_is_supported = input.config.rows == 3 && input.config.cols == 3;
  let held_mask_is_valid = (input.held_cols_mask & !0b0000_0111) == 0;
  let held_context = collect_held_spin_context(input.held_cols_mask, input.previous_grid);

  let mut rng = ChaCha20Rng::from_seed(input.seed_ctx.round_seed);
  let mut current_grid = build_initial_grid(&input, &held_context, &mut rng);
  let initial_grid = current_grid;

  if !layout_is_supported || !held_mask_is_valid {
    return SpinEngineOutput {
      initial_grid,
      final_grid: current_grid,
      cascades: Vec::new(),
      total_win: 0,
      total_multiplier_10000x: 0,
      free_spins_awarded: 0,
      jackpot_hit: false,
      jackpot_line_count: 0,
    };
  }

  let mut cascades = Vec::new();
  let mut total_win = 0u64;
  let mut total_free_spins = 0u16;
  let mut jackpot_hit = false;
  let mut jackpot_line_count = 0u8;
  let mut step_index = 0u8;

  while step_index < input.config.max_cascade_steps {
    let step_multiplier = input
      .config
      .cascade_multiplier_table
      .get(step_index as usize)
      .copied()
      .unwrap_or_else(|| input.config.cascade_multiplier_table.last().copied().unwrap_or(10_000));
    let winning_lines = evaluate_winning_lines(&current_grid, step_multiplier, input.bet, &input.config);
    if winning_lines.is_empty() {
      break;
    }

    let step_win = winning_lines.iter().fold(0u64, |sum, line| sum.saturating_add(line.line_win));
    total_win = total_win.saturating_add(step_win);

    let awarded_free_spins = resolve_awarded_free_spins(&winning_lines, &input.config);
    total_free_spins = total_free_spins.saturating_add(awarded_free_spins);

    let contains_jackpot_line = winning_lines.iter().any(|line| line.is_jackpot_line);
    if contains_jackpot_line {
      jackpot_hit = true;
      let jackpot_hits = winning_lines.iter().filter(|line| line.is_jackpot_line).count();
      let jackpot_hits = jackpot_hits.min(u8::MAX as usize) as u8;
      jackpot_line_count = jackpot_line_count.saturating_add(jackpot_hits);
      cascades.push(CascadeResolution {
        step_index,
        multiplier_10000x: step_multiplier,
        grid_after_step: current_grid,
        winning_lines,
        step_win,
        awarded_free_spins,
        contains_jackpot_line,
      });
      break;
    }

    current_grid = apply_cascade(current_grid, &winning_lines, &input.config, &mut rng);

    cascades.push(CascadeResolution {
      step_index,
      multiplier_10000x: step_multiplier,
      grid_after_step: current_grid,
      winning_lines,
      step_win,
      awarded_free_spins,
      contains_jackpot_line,
    });

    step_index = step_index.saturating_add(1);
  }

  let total_multiplier_10000x = if input.bet == 0 {
    0
  } else {
    saturating_u128_to_u32((total_win as u128) * 10_000u128 / input.bet as u128)
  };

  SpinEngineOutput {
    initial_grid,
    final_grid: current_grid,
    cascades,
    total_win,
    total_multiplier_10000x,
    free_spins_awarded: total_free_spins,
    jackpot_hit,
    jackpot_line_count,
  }
}

fn build_initial_grid(input: &SpinEngineInput, held_context: &HeldSpinContext, rng: &mut ChaCha20Rng) -> [SymbolCode; 9] {
  let mut grid = [0u8; 9];
  for index in 0..9 {
    let col = (index % 3) as u8;
    let should_hold = (input.held_cols_mask & (1 << col)) != 0;
    if should_hold {
      if let Some(previous) = input.previous_grid {
        grid[index] = previous[index];
        continue;
      }
    }
    grid[index] = sample_symbol_for_initial_grid(&input.config, held_context, rng);
  }
  grid
}

fn collect_held_spin_context(held_cols_mask: u8, previous_grid: Option<[SymbolCode; 9]>) -> HeldSpinContext {
  let held_count = held_cols_mask.count_ones() as u8;
  if held_count == 0 {
    return HeldSpinContext::default();
  }

  let Some(previous_grid) = previous_grid else {
    return HeldSpinContext::default();
  };

  let mut held_symbols = HashSet::new();
  for col in 0..3u8 {
    if (held_cols_mask & (1 << col)) == 0 {
      continue;
    }

    for row in 0..3usize {
      let index = row * 3 + col as usize;
      held_symbols.insert(previous_grid[index]);
    }
  }

  HeldSpinContext {
    held_symbols,
    held_count,
  }
}

fn sample_symbol_for_initial_grid(config: &SpinAlgorithmConfig, held_context: &HeldSpinContext, rng: &mut ChaCha20Rng) -> SymbolCode {
  if held_context.held_count == 0 || config.hold_damping_scalar_10000x == 0 {
    return sample_symbol(config, rng);
  }

  if config.symbol_weights.is_empty() {
    return 0;
  }

  let damping_power = held_context.held_count as f64 * config.hold_damping_scalar_10000x as f64 / 10_000.0;
  let mut total_weight = 0.0f64;
  let mut dynamic_weights = Vec::with_capacity(config.symbol_weights.len());

  for item in &config.symbol_weights {
    let mut weight = item.weight_ppm as f64;
    let is_special_symbol = config.jackpot_symbols.contains(&item.symbol_code) || item.symbol_code == config.wild_symbol;
    if held_context.held_symbols.contains(&item.symbol_code) || is_special_symbol {
      let payout_multiplier = find_payout_rule(config, item.symbol_code)
        .map(|rule| rule.payout_multiplier_10000x as f64 / 10_000.0)
        .unwrap_or(0.0)
        .max(2.0);
      let dampening_factor = payout_multiplier.powf(damping_power);
      if dampening_factor.is_finite() && dampening_factor > 0.0 {
        weight /= dampening_factor;
      }
    }

    total_weight += weight;
    dynamic_weights.push((item.symbol_code, weight));
  }

  if total_weight <= 0.0 || !total_weight.is_finite() {
    return config.symbol_weights[0].symbol_code;
  }

  let target = rng.random_range(0.0..total_weight);
  let mut acc = 0.0;
  for (symbol_code, weight) in dynamic_weights {
    acc += weight;
    if target <= acc {
      return symbol_code;
    }
  }

  config.symbol_weights.last().map(|item| item.symbol_code).unwrap_or(0)
}

fn sample_symbol(config: &SpinAlgorithmConfig, rng: &mut ChaCha20Rng) -> SymbolCode {
  if config.symbol_weights.is_empty() {
    return 0;
  }

  let total_weight = config.symbol_weights.iter().map(|item| item.weight_ppm as u64).sum::<u64>();
  if total_weight == 0 {
    return config.symbol_weights[0].symbol_code;
  }

  let target = rng.random_range(0..total_weight);
  let mut acc = 0u64;
  for item in &config.symbol_weights {
    acc = acc.saturating_add(item.weight_ppm as u64);
    if target < acc {
      return item.symbol_code;
    }
  }

  config.symbol_weights.last().map(|item| item.symbol_code).unwrap_or(0)
}

fn evaluate_winning_lines(
  grid: &[SymbolCode; 9],
  cascade_multiplier_10000x: u32,
  bet: u64,
  config: &SpinAlgorithmConfig,
) -> Vec<WinningLineResolution> {
  let mut results = Vec::new();

  for (payline_index, cells) in config.paylines.iter().enumerate() {
    let Some(line_symbols) = safe_line_symbols(grid, cells) else {
      continue;
    };

    let target_symbol = resolve_target_symbol(&line_symbols, config.wild_symbol);

    if !line_matches(&line_symbols, target_symbol, config.wild_symbol) {
      continue;
    }

    let Some(payout_rule) = find_payout_rule(config, target_symbol) else {
      continue;
    };

    let is_jackpot_line = payout_rule.is_jackpot_symbol || config.jackpot_symbols.contains(&target_symbol);
    let line_multiplier_10000x = if is_jackpot_line {
      payout_rule.payout_multiplier_10000x
    } else {
      saturating_u128_to_u32((payout_rule.payout_multiplier_10000x as u128 * cascade_multiplier_10000x as u128) / 10_000u128)
    };

    let line_win = saturating_u128_to_u64((bet as u128) * line_multiplier_10000x as u128 / 10_000u128);
    results.push(WinningLineResolution {
      payline_id: payline_index.min(u8::MAX as usize) as u8,
      symbol_code: target_symbol,
      cells: *cells,
      line_multiplier_10000x,
      line_win,
      is_jackpot_line,
    });
  }

  results
}

fn safe_line_symbols(grid: &[SymbolCode; 9], cells: &[u8; 3]) -> Option<[SymbolCode; 3]> {
  let [c0, c1, c2] = *cells;
  let i0 = c0 as usize;
  let i1 = c1 as usize;
  let i2 = c2 as usize;
  if i0 >= grid.len() || i1 >= grid.len() || i2 >= grid.len() {
    return None;
  }

  Some([grid[i0], grid[i1], grid[i2]])
}

fn resolve_target_symbol(line_symbols: &[SymbolCode; 3], wild_symbol: SymbolCode) -> SymbolCode {
  line_symbols.iter().copied().find(|symbol| *symbol != wild_symbol).unwrap_or(wild_symbol)
}

fn line_matches(line_symbols: &[SymbolCode; 3], target_symbol: SymbolCode, wild_symbol: SymbolCode) -> bool {
  line_symbols.iter().all(|symbol| *symbol == target_symbol || *symbol == wild_symbol)
}

fn find_payout_rule(config: &SpinAlgorithmConfig, symbol_code: SymbolCode) -> Option<&AlgorithmPayoutRule> {
  config
    .payout_rules
    .iter()
    .find(|rule| rule.symbol_code == symbol_code && rule.match_count == 3)
}

fn resolve_awarded_free_spins(winning_lines: &[WinningLineResolution], config: &SpinAlgorithmConfig) -> u16 {
  let mut best_award = 0u16;

  for rule in &config.free_spin_rules {
    let line_hits = winning_lines.iter().filter(|line| line.symbol_code == rule.trigger_symbol).count() as u8;
    if line_hits >= rule.min_line_hits {
      best_award = best_award.max(rule.award_spins);
    }
  }

  best_award
}

fn apply_cascade(
  current_grid: [SymbolCode; 9],
  winning_lines: &[WinningLineResolution],
  config: &SpinAlgorithmConfig,
  rng: &mut ChaCha20Rng,
) -> [SymbolCode; 9] {
  let mut removable = [false; 9];
  for line in winning_lines.iter().filter(|line| !line.is_jackpot_line) {
    for cell in line.cells {
      let index = cell as usize;
      if index < removable.len() {
        removable[index] = true;
      }
    }
  }

  let mut next_grid = current_grid;
  for col in 0..3usize {
    let indices = [col, col + 3, col + 6];
    let remaining = indices
      .iter()
      .copied()
      .filter(|index| !removable[*index])
      .map(|index| current_grid[index])
      .collect::<Vec<_>>();
    let refill_count = 3usize.saturating_sub(remaining.len());

    let mut rebuilt = Vec::with_capacity(3);
    for _ in 0..refill_count {
      rebuilt.push(sample_symbol(config, rng));
    }
    rebuilt.extend(remaining);

    for (row, index) in indices.iter().enumerate() {
      next_grid[*index] = rebuilt[row];
    }
  }

  next_grid
}

fn saturating_u128_to_u32(value: u128) -> u32 {
  value.min(u32::MAX as u128) as u32
}

fn saturating_u128_to_u64(value: u128) -> u64 {
  value.min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::fruits_garden::paylines::DEFAULT_PAYLINES;
  use crate::fruits_garden::types::{
    AlgorithmFreeSpinRule, AlgorithmPayoutRule, AlgorithmSymbolWeight, RoundSeedContext, SpinAlgorithmConfig, SpinEngineInput,
  };

  fn config() -> SpinAlgorithmConfig {
    SpinAlgorithmConfig {
      rows: 3,
      cols: 3,
      max_cascade_steps: 5,
      cascade_multiplier_table: vec![10_000, 20_000, 30_000, 40_000, 50_000],
      hold_damping_scalar_10000x: 0,
      symbol_weights: vec![
        AlgorithmSymbolWeight {
          symbol_code: 0,
          weight_ppm: 10_000,
        },
        AlgorithmSymbolWeight {
          symbol_code: 9,
          weight_ppm: 1_000,
        },
        AlgorithmSymbolWeight {
          symbol_code: 10,
          weight_ppm: 500,
        },
      ],
      paylines: DEFAULT_PAYLINES.to_vec(),
      payout_rules: vec![
        AlgorithmPayoutRule {
          symbol_code: 0,
          match_count: 3,
          payout_multiplier_10000x: 10_000,
          is_jackpot_symbol: false,
        },
        AlgorithmPayoutRule {
          symbol_code: 9,
          match_count: 3,
          payout_multiplier_10000x: 20_000,
          is_jackpot_symbol: true,
        },
        AlgorithmPayoutRule {
          symbol_code: 10,
          match_count: 3,
          payout_multiplier_10000x: 30_000,
          is_jackpot_symbol: true,
        },
      ],
      free_spin_rules: vec![AlgorithmFreeSpinRule {
        trigger_symbol: 0,
        min_line_hits: 2,
        award_spins: 3,
      }],
      jackpot_symbols: vec![9, 10],
      wild_symbol: 10,
    }
  }

  #[test]
  fn deterministic_with_same_seed() {
    let input = SpinEngineInput {
      bet: 100,
      held_cols_mask: 0,
      previous_grid: None,
      config: config(),
      seed_ctx: RoundSeedContext {
        root_seed: [1u8; 32],
        round_seed: [2u8; 32],
        vrf_seed_idx: Some(1),
      },
    };

    let result_a = execute_spin(input.clone());
    let result_b = execute_spin(input);
    assert_eq!(result_a, result_b);
  }

  #[test]
  fn held_columns_are_preserved_in_initial_grid() {
    let input = SpinEngineInput {
      bet: 100,
      held_cols_mask: 0b010,
      previous_grid: Some([1, 2, 3, 4, 5, 6, 7, 8, 9]),
      config: config(),
      seed_ctx: RoundSeedContext {
        root_seed: [3u8; 32],
        round_seed: [4u8; 32],
        vrf_seed_idx: Some(2),
      },
    };

    let result = execute_spin(input);
    assert_eq!(result.initial_grid[1], 2);
    assert_eq!(result.initial_grid[4], 5);
    assert_eq!(result.initial_grid[7], 8);
  }

  #[test]
  fn invalid_layout_returns_empty_result() {
    let mut cfg = config();
    cfg.rows = 4;

    let input = SpinEngineInput {
      bet: 100,
      held_cols_mask: 0,
      previous_grid: None,
      config: cfg,
      seed_ctx: RoundSeedContext {
        root_seed: [5u8; 32],
        round_seed: [6u8; 32],
        vrf_seed_idx: Some(3),
      },
    };

    let result = execute_spin(input);
    assert!(result.cascades.is_empty());
    assert_eq!(result.total_win, 0);
    assert_eq!(result.total_multiplier_10000x, 0);
    assert_eq!(result.final_grid, result.initial_grid);
  }

  #[test]
  fn invalid_payline_indices_are_ignored() {
    let mut cfg = config();
    cfg.max_cascade_steps = 1;
    cfg.paylines = vec![[0, 1, 99], [0, 1, 2]];
    cfg.symbol_weights = vec![AlgorithmSymbolWeight {
      symbol_code: 1,
      weight_ppm: 1,
    }];
    cfg.payout_rules = vec![AlgorithmPayoutRule {
      symbol_code: 1,
      match_count: 3,
      payout_multiplier_10000x: 10_000,
      is_jackpot_symbol: false,
    }];
    cfg.jackpot_symbols = Vec::new();
    cfg.wild_symbol = 10;

    let input = SpinEngineInput {
      bet: 100,
      held_cols_mask: 0b111,
      previous_grid: Some([1; 9]),
      config: cfg,
      seed_ctx: RoundSeedContext {
        root_seed: [7u8; 32],
        round_seed: [8u8; 32],
        vrf_seed_idx: Some(4),
      },
    };

    let result = execute_spin(input);
    assert_eq!(result.cascades.len(), 1);
    assert_eq!(result.cascades[0].winning_lines.len(), 1);
    assert_eq!(result.cascades[0].winning_lines[0].payline_id, 1);
  }

  #[test]
  fn arithmetic_uses_saturating_conversions() {
    let mut cfg = config();
    cfg.max_cascade_steps = 1;
    cfg.cascade_multiplier_table = vec![10_000];
    cfg.symbol_weights = vec![AlgorithmSymbolWeight {
      symbol_code: 1,
      weight_ppm: 1,
    }];
    cfg.payout_rules = vec![AlgorithmPayoutRule {
      symbol_code: 1,
      match_count: 3,
      payout_multiplier_10000x: u32::MAX,
      is_jackpot_symbol: false,
    }];
    cfg.jackpot_symbols = Vec::new();
    cfg.wild_symbol = 10;

    let huge_bet_input = SpinEngineInput {
      bet: u64::MAX,
      held_cols_mask: 0b111,
      previous_grid: Some([1; 9]),
      config: cfg.clone(),
      seed_ctx: RoundSeedContext {
        root_seed: [9u8; 32],
        round_seed: [10u8; 32],
        vrf_seed_idx: Some(5),
      },
    };

    let huge_bet_result = execute_spin(huge_bet_input);
    assert_eq!(huge_bet_result.cascades[0].step_win, u64::MAX);
    assert_eq!(huge_bet_result.total_win, u64::MAX);

    let small_bet_input = SpinEngineInput {
      bet: 1,
      held_cols_mask: 0b111,
      previous_grid: Some([1; 9]),
      config: cfg,
      seed_ctx: RoundSeedContext {
        root_seed: [11u8; 32],
        round_seed: [12u8; 32],
        vrf_seed_idx: Some(6),
      },
    };

    let small_bet_result = execute_spin(small_bet_input);
    assert_eq!(small_bet_result.total_multiplier_10000x, u32::MAX);
  }

  #[test]
  fn hold_damping_is_noop_without_holds() {
    let mut no_damping = config();
    no_damping.hold_damping_scalar_10000x = 0;

    let mut with_damping = config();
    with_damping.hold_damping_scalar_10000x = 8_000;

    let base_input = SpinEngineInput {
      bet: 100,
      held_cols_mask: 0,
      previous_grid: None,
      config: no_damping,
      seed_ctx: RoundSeedContext {
        root_seed: [13u8; 32],
        round_seed: [14u8; 32],
        vrf_seed_idx: Some(7),
      },
    };

    let damped_input = SpinEngineInput {
      config: with_damping,
      ..base_input.clone()
    };

    let base_result = execute_spin(base_input);
    let damped_result = execute_spin(damped_input);
    assert_eq!(base_result, damped_result);
  }

  #[test]
  fn hold_damping_reduces_held_symbol_frequency_on_initial_non_held_cells() {
    let mut baseline_cfg = config();
    baseline_cfg.hold_damping_scalar_10000x = 0;
    baseline_cfg.symbol_weights = vec![
      AlgorithmSymbolWeight {
        symbol_code: 9,
        weight_ppm: 3_000,
      },
      AlgorithmSymbolWeight {
        symbol_code: 1,
        weight_ppm: 3_000,
      },
      AlgorithmSymbolWeight {
        symbol_code: 10,
        weight_ppm: 1_000,
      },
    ];

    baseline_cfg.payout_rules = vec![
      AlgorithmPayoutRule {
        symbol_code: 1,
        match_count: 3,
        payout_multiplier_10000x: 10_000,
        is_jackpot_symbol: false,
      },
      AlgorithmPayoutRule {
        symbol_code: 9,
        match_count: 3,
        payout_multiplier_10000x: 80_000,
        is_jackpot_symbol: true,
      },
      AlgorithmPayoutRule {
        symbol_code: 10,
        match_count: 3,
        payout_multiplier_10000x: 100_000,
        is_jackpot_symbol: true,
      },
    ];
    baseline_cfg.jackpot_symbols = vec![9, 10];

    let mut damped_cfg = baseline_cfg.clone();
    damped_cfg.hold_damping_scalar_10000x = 8_000;

    let previous_grid = [9, 1, 1, 9, 1, 1, 9, 1, 1];
    let mut baseline_hits = 0u32;
    let mut damped_hits = 0u32;

    for seed_index in 0u16..240u16 {
      let mut round_seed = [0u8; 32];
      round_seed[0] = (seed_index & 0x00ff) as u8;
      round_seed[1] = (seed_index >> 8) as u8;

      let baseline_result = execute_spin(SpinEngineInput {
        bet: 100,
        held_cols_mask: 0b001,
        previous_grid: Some(previous_grid),
        config: baseline_cfg.clone(),
        seed_ctx: RoundSeedContext {
          root_seed: [15u8; 32],
          round_seed,
          vrf_seed_idx: Some(seed_index as u64),
        },
      });

      let damped_result = execute_spin(SpinEngineInput {
        bet: 100,
        held_cols_mask: 0b001,
        previous_grid: Some(previous_grid),
        config: damped_cfg.clone(),
        seed_ctx: RoundSeedContext {
          root_seed: [15u8; 32],
          round_seed,
          vrf_seed_idx: Some(seed_index as u64),
        },
      });

      for &index in &[1usize, 2usize, 4usize, 5usize, 7usize, 8usize] {
        if baseline_result.initial_grid[index] == 9 {
          baseline_hits = baseline_hits.saturating_add(1);
        }
        if damped_result.initial_grid[index] == 9 {
          damped_hits = damped_hits.saturating_add(1);
        }
      }
    }

    assert!(
      damped_hits < baseline_hits,
      "expected damped hits ({}) to be lower than baseline hits ({})",
      damped_hits,
      baseline_hits
    );
  }
}
