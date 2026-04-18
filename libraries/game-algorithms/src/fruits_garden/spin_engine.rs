use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

use super::types::{
  AlgorithmPayoutRule, CascadeResolution, SpinAlgorithmConfig, SpinEngineInput, SpinEngineOutput, SymbolCode, WinningLineResolution,
};

pub fn execute_spin(input: SpinEngineInput) -> SpinEngineOutput {
  let mut rng = ChaCha20Rng::from_seed(input.seed_ctx.round_seed);
  let mut current_grid = build_initial_grid(&input, &mut rng);
  let initial_grid = current_grid;

  let mut cascades = Vec::new();
  let mut total_win = 0u64;
  let mut total_free_spins = 0u16;
  let mut jackpot_hit = false;
  let mut jackpot_line_count = 0u8;
  let mut step_index = 0u8;
  let mut step_multiplier = 10_000u32;

  while step_index < input.config.max_cascade_steps {
    let winning_lines = evaluate_winning_lines(&current_grid, step_multiplier, input.bet, &input.config);
    if winning_lines.is_empty() {
      break;
    }

    let step_win = winning_lines.iter().map(|line| line.line_win).sum::<u64>();
    total_win = total_win.saturating_add(step_win);

    let awarded_free_spins = resolve_awarded_free_spins(&winning_lines, &input.config);
    total_free_spins = total_free_spins.saturating_add(awarded_free_spins);

    let contains_jackpot_line = winning_lines.iter().any(|line| line.is_jackpot_line);
    if contains_jackpot_line {
      jackpot_hit = true;
      jackpot_line_count = jackpot_line_count.saturating_add(winning_lines.iter().filter(|line| line.is_jackpot_line).count() as u8);
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
    step_multiplier = step_multiplier.saturating_add(10_000);
  }

  let total_multiplier_10000x = if input.bet == 0 {
    0
  } else {
    ((total_win as u128) * 10_000u128 / input.bet as u128) as u32
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

fn build_initial_grid(input: &SpinEngineInput, rng: &mut ChaCha20Rng) -> [SymbolCode; 9] {
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
    grid[index] = sample_symbol(&input.config, rng);
  }
  grid
}

fn sample_symbol(config: &SpinAlgorithmConfig, rng: &mut ChaCha20Rng) -> SymbolCode {
  if config.symbol_weights.is_empty() {
    return 0;
  }

  let total_weight = config.symbol_weights.iter().map(|item| item.weight_ppm as u64).sum::<u64>();
  if total_weight == 0 {
    return config.symbol_weights[0].symbol_code;
  }

  let target = (rng.next_u32() as u64) % total_weight;
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
    let line_symbols = [grid[cells[0] as usize], grid[cells[1] as usize], grid[cells[2] as usize]];
    let Some(target_symbol) = resolve_target_symbol(&line_symbols, config.wild_symbol) else {
      continue;
    };

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
      ((payout_rule.payout_multiplier_10000x as u128 * cascade_multiplier_10000x as u128) / 10_000u128) as u32
    };

    let line_win = ((bet as u128) * line_multiplier_10000x as u128 / 10_000u128) as u64;
    results.push(WinningLineResolution {
      payline_id: payline_index as u8,
      symbol_code: target_symbol,
      cells: *cells,
      line_multiplier_10000x,
      line_win,
      is_jackpot_line,
    });
  }

  results
}

fn resolve_target_symbol(line_symbols: &[SymbolCode; 3], wild_symbol: SymbolCode) -> Option<SymbolCode> {
  line_symbols.iter().copied().find(|symbol| *symbol != wild_symbol).or(Some(wild_symbol))
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
      removable[cell as usize] = true;
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
}
