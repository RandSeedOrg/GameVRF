use std::collections::HashSet;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub fn compute_mine_positions(seed: [u8; 32], total_cells: u16, mines_count: u16) -> HashSet<u16> {
    if total_cells == 0 || mines_count == 0 {
        return HashSet::new();
    }

    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut positions: Vec<u16> = (0..total_cells).collect();
    let k = mines_count.min(total_cells) as usize;

    for i in 0..k {
        let j = rng.random_range(i..positions.len());
        positions.swap(i, j);
    }

    positions
        .into_iter()
        .take(k)
        .collect()
}

pub fn mine_positions_sorted(seed: [u8; 32], total_cells: u16, mines_count: u16) -> Vec<u16> {
    let mut positions: Vec<u16> = compute_mine_positions(seed, total_cells, mines_count)
        .into_iter()
        .collect();
    positions.sort_unstable();
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mine_count_is_exact() {
        let seed = [42u8; 32];
        let mines = compute_mine_positions(seed, 25, 5);
        assert_eq!(mines.len(), 5);
    }

    #[test]
    fn all_positions_in_range() {
        let seed = [7u8; 32];
        let mines = compute_mine_positions(seed, 25, 10);
        for &m in &mines {
            assert!(m < 25, "mine {} out of range", m);
        }
    }

    #[test]
    fn deterministic() {
        let seed = [1u8; 32];
        let a = compute_mine_positions(seed, 25, 3);
        let b = compute_mine_positions(seed, 25, 3);
        assert_eq!(a, b);
    }

    #[test]
    fn sorted_positions_ascending() {
        let seed = [99u8; 32];
        let sorted = mine_positions_sorted(seed, 25, 8);
        assert_eq!(sorted.len(), 8);
        for w in sorted.windows(2) {
            assert!(w[0] < w[1]);
        }
    }
}
