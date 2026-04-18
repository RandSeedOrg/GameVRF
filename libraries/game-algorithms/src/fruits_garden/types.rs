pub type SymbolCode = u8;
pub type PaylineId = u8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgorithmSymbolWeight {
  pub symbol_code: SymbolCode,
  pub weight_ppm: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgorithmPayoutRule {
  pub symbol_code: SymbolCode,
  pub match_count: u8,
  pub payout_multiplier_10000x: u32,
  pub is_jackpot_symbol: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgorithmFreeSpinRule {
  pub trigger_symbol: SymbolCode,
  pub min_line_hits: u8,
  pub award_spins: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpinAlgorithmConfig {
  pub rows: u8,
  pub cols: u8,
  pub max_cascade_steps: u8,
  pub symbol_weights: Vec<AlgorithmSymbolWeight>,
  pub paylines: Vec<[u8; 3]>,
  pub payout_rules: Vec<AlgorithmPayoutRule>,
  pub free_spin_rules: Vec<AlgorithmFreeSpinRule>,
  pub jackpot_symbols: Vec<SymbolCode>,
  pub wild_symbol: SymbolCode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoundSeedContext {
  pub root_seed: [u8; 32],
  pub round_seed: [u8; 32],
  pub vrf_seed_idx: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WinningLineResolution {
  pub payline_id: PaylineId,
  pub symbol_code: SymbolCode,
  pub cells: [u8; 3],
  pub line_multiplier_10000x: u32,
  pub line_win: u64,
  pub is_jackpot_line: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CascadeResolution {
  pub step_index: u8,
  pub multiplier_10000x: u32,
  pub grid_after_step: [SymbolCode; 9],
  pub winning_lines: Vec<WinningLineResolution>,
  pub step_win: u64,
  pub awarded_free_spins: u16,
  pub contains_jackpot_line: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpinEngineInput {
  pub bet: u64,
  pub held_cols_mask: u8,
  pub previous_grid: Option<[SymbolCode; 9]>,
  pub config: SpinAlgorithmConfig,
  pub seed_ctx: RoundSeedContext,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpinEngineOutput {
  pub initial_grid: [SymbolCode; 9],
  pub final_grid: [SymbolCode; 9],
  pub cascades: Vec<CascadeResolution>,
  pub total_win: u64,
  pub total_multiplier_10000x: u32,
  pub free_spins_awarded: u16,
  pub jackpot_hit: bool,
  pub jackpot_line_count: u8,
}
