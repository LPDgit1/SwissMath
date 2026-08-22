use core::fmt;

use crate::{
    LinearCongruence, LinearSolution, Modulus, ResidueError, ResidueSet, solve_linear_congruence,
};

/// A finite modular filter `x mod m ∈ allowed`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModularFilter {
    modulus: Modulus,
    allowed: ResidueSet,
}

impl ModularFilter {
    /// Creates an allowed-residue filter.
    pub fn from_allowed<I>(modulus: Modulus, residues: I) -> Result<Self, ResidueError>
    where
        I: IntoIterator<Item = u64>,
    {
        Ok(Self {
            modulus,
            allowed: ResidueSet::from_iter(modulus, residues)?,
        })
    }

    /// Creates an excluded-residue filter by complementing the excluded set.
    pub fn from_excluded<I>(modulus: Modulus, residues: I) -> Result<Self, ResidueError>
    where
        I: IntoIterator<Item = u64>,
    {
        let mut allowed = ResidueSet::from_iter(modulus, residues)?;
        allowed.complement_assign();
        Ok(Self { modulus, allowed })
    }

    /// Converts a linear congruence directly into its reduced modular filter.
    #[must_use]
    pub fn from_linear_congruence(equation: LinearCongruence) -> ModularFilterBuild {
        match solve_linear_congruence(equation).solution {
            LinearSolution::None => ModularFilterBuild::None,
            LinearSolution::All => ModularFilterBuild::All,
            LinearSolution::Class(congruence) => {
                let residue = congruence.residue();
                let modulus = congruence.modulus();
                ModularFilterBuild::Filter(
                    Self::from_allowed(modulus, [residue]).expect("singleton is valid"),
                )
            }
        }
    }

    /// Returns the filter modulus.
    #[inline]
    #[must_use]
    pub const fn modulus(&self) -> Modulus {
        self.modulus
    }

    /// Returns the allowed residues.
    #[inline]
    #[must_use]
    pub fn allowed(&self) -> &ResidueSet {
        &self.allowed
    }

    /// Returns whether this filter allows no values.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.allowed.is_empty()
    }

    /// Returns whether this filter is tautological.
    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.allowed.is_full()
    }
}

/// Outcome of converting a linear congruence into a sieve filter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModularFilterBuild {
    /// The complete sieve is contradictory.
    None,
    /// The congruence imposes no restriction.
    All,
    /// A compact reduced filter.
    Filter(ModularFilter),
}

/// Errors from inclusive finite-range sieve searches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SieveError {
    /// The inclusive range has `start > end`.
    InvalidRange,
    /// A filter could not be materialized.
    Residue(ResidueError),
}

impl fmt::Display for SieveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange => f.write_str("the inclusive range must satisfy start <= end"),
            Self::Residue(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for SieveError {}

impl From<ResidueError> for SieveError {
    fn from(error: ResidueError) -> Self {
        Self::Residue(error)
    }
}

/// Statistics and an optional ascending preview returned by a sieve search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SieveResult {
    pub start: u64,
    pub end: u64,
    pub total_values: u128,
    pub normalized_filter_count: usize,
    pub survivor_count: u128,
    pub preview: Vec<u64>,
    pub anchor_modulus: Option<Modulus>,
    pub anchor_allowed_count: u64,
}

/// A normalized collection of modular filters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModularSieve {
    filters: Vec<ModularFilter>,
    impossible: bool,
}

impl ModularSieve {
    /// Normalizes same-modulus filters, removes full filters, and detects emptiness.
    pub fn new<I>(filters: I) -> Result<Self, SieveError>
    where
        I: IntoIterator<Item = ModularFilter>,
    {
        let mut normalized = Vec::new();
        for filter in filters {
            if filter.is_empty() {
                return Ok(Self {
                    filters: Vec::new(),
                    impossible: true,
                });
            }
            if filter.is_full() {
                continue;
            }
            if let Some(existing) = normalized
                .iter_mut()
                .find(|existing: &&mut ModularFilter| existing.modulus == filter.modulus)
            {
                existing
                    .allowed
                    .intersect_assign(&filter.allowed)
                    .map_err(SieveError::Residue)?;
                if existing.is_empty() {
                    return Ok(Self {
                        filters: Vec::new(),
                        impossible: true,
                    });
                }
            } else {
                normalized.push(filter);
            }
        }

        normalized.sort_unstable_by(|left, right| {
            let left_weight = u128::from(left.allowed.len()) * u128::from(right.modulus.get());
            let right_weight = u128::from(right.allowed.len()) * u128::from(left.modulus.get());
            left_weight
                .cmp(&right_weight)
                .then_with(|| left.modulus.get().cmp(&right.modulus.get()))
        });

        Ok(Self {
            filters: normalized,
            impossible: false,
        })
    }

    /// Returns the number of filters after normalization.
    #[inline]
    #[must_use]
    pub fn normalized_filter_count(&self) -> usize {
        self.filters.len()
    }

    /// Searches the inclusive range `start ..= end` and retains at most
    /// `preview_limit` ascending matches.
    pub fn search(
        &self,
        start: u64,
        end: u64,
        preview_limit: usize,
    ) -> Result<SieveResult, SieveError> {
        if start > end {
            return Err(SieveError::InvalidRange);
        }

        let total_values = u128::from(end) - u128::from(start) + 1;
        if self.impossible {
            return Ok(SieveResult {
                start,
                end,
                total_values,
                normalized_filter_count: 0,
                survivor_count: 0,
                preview: Vec::new(),
                anchor_modulus: None,
                anchor_allowed_count: 0,
            });
        }

        if self.filters.is_empty() {
            let mut preview = Vec::with_capacity(preview_limit.min(1024));
            let mut candidate = start;
            while preview.len() < preview_limit {
                preview.push(candidate);
                if candidate == end {
                    break;
                }
                candidate += 1;
            }
            return Ok(SieveResult {
                start,
                end,
                total_values,
                normalized_filter_count: 0,
                survivor_count: total_values,
                preview,
                anchor_modulus: None,
                anchor_allowed_count: 0,
            });
        }

        let anchor = &self.filters[0];
        let anchor_modulus = anchor.modulus.get();
        let anchor_allowed_count = anchor.allowed.len();
        let last_block = end / anchor_modulus;
        let mut block = start / anchor_modulus;
        let mut survivor_count = 0_u128;
        let mut preview = Vec::with_capacity(preview_limit.min(1024));

        loop {
            let block_base = block
                .checked_mul(anchor_modulus)
                .expect("block base is at most end");
            for residue in anchor.allowed.iter() {
                let Some(candidate) = block_base.checked_add(residue) else {
                    continue;
                };
                if candidate < start || candidate > end {
                    continue;
                }
                if self.filters[1..]
                    .iter()
                    .all(|filter| filter.allowed.contains(candidate % filter.modulus.get()))
                {
                    survivor_count += 1;
                    if preview.len() < preview_limit {
                        preview.push(candidate);
                    }
                }
            }
            if block == last_block {
                break;
            }
            block += 1;
        }

        Ok(SieveResult {
            start,
            end,
            total_values,
            normalized_filter_count: self.filters.len(),
            survivor_count,
            preview,
            anchor_modulus: Some(anchor.modulus),
            anchor_allowed_count,
        })
    }
}
