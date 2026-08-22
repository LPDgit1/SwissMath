use core::{fmt, iter::FusedIterator, mem::size_of};

use crate::{Modulus, bitops};

// Selected by the Phase 0 threshold benchmark. This is deliberately private.
const INLINE_WORDS: usize = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
enum ResidueStorage {
    Inline([u64; INLINE_WORDS]),
    Heap(Vec<u64>),
}

/// Error returned by materialized residue-set operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResidueError {
    /// A residue is outside `[0, modulus)`.
    OutOfRange,
    /// Binary operands have different moduli.
    ModulusMismatch,
    /// The requested materialized storage could not be allocated.
    AllocationFailed,
}

impl fmt::Display for ResidueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange => f.write_str("residue is outside the modulus"),
            Self::ModulusMismatch => f.write_str("residue-set moduli differ"),
            Self::AllocationFailed => f.write_str("residue-set allocation failed"),
        }
    }
}

impl std::error::Error for ResidueError {}

/// Returns the logical heap bytes required by a materialized residue set.
///
/// The result is zero when the private inline representation is sufficient.
#[must_use]
pub fn required_heap_bytes(modulus: Modulus) -> usize {
    let words = word_count_u64(modulus);
    if words <= INLINE_WORDS as u64 {
        0
    } else {
        // Official targets are 64-bit, so every possible word count and byte
        // count for a u64 modulus is representable in usize.
        words as usize * size_of::<u64>()
    }
}

/// A materialized subset of `Z / mZ` stored as contiguous logical `u64` words.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidueSet {
    modulus: Modulus,
    len: u64,
    storage: ResidueStorage,
}

impl ResidueSet {
    /// Constructs the empty set.
    pub fn try_empty(modulus: Modulus) -> Result<Self, ResidueError> {
        Self::new_zeroed(modulus)
    }

    /// Constructs the full set.
    pub fn try_full(modulus: Modulus) -> Result<Self, ResidueError> {
        let mut set = Self::new_zeroed(modulus)?;
        set.fill_full();
        set.assert_invariants();
        Ok(set)
    }

    /// Constructs a singleton, rejecting an out-of-range residue.
    pub fn singleton(modulus: Modulus, residue: u64) -> Result<Self, ResidueError> {
        if residue >= modulus.get() {
            return Err(ResidueError::OutOfRange);
        }
        let mut set = Self::new_zeroed(modulus)?;
        let index = (residue / 64) as usize;
        let mask = 1_u64 << (residue % 64);
        set.words_mut()[index] = mask;
        set.len = 1;
        set.assert_invariants();
        Ok(set)
    }

    /// Materializes canonical residues from an iterator.
    pub fn from_iter<I>(modulus: Modulus, values: I) -> Result<Self, ResidueError>
    where
        I: IntoIterator<Item = u64>,
    {
        let mut set = Self::new_zeroed(modulus)?;
        let limit = modulus.get();
        let mut len = 0_u64;
        {
            let words = set.words_mut();
            for residue in values {
                if residue >= limit {
                    return Err(ResidueError::OutOfRange);
                }
                let word = &mut words[(residue / 64) as usize];
                let mask = 1_u64 << (residue % 64);
                if *word & mask == 0 {
                    *word |= mask;
                    len += 1;
                }
            }
        }
        set.len = len;
        set.assert_invariants();
        Ok(set)
    }

    /// Materializes exactly the residues accepted by `predicate`.
    pub fn from_predicate<F>(modulus: Modulus, mut predicate: F) -> Result<Self, ResidueError>
    where
        F: FnMut(u64) -> bool,
    {
        let mut set = Self::new_zeroed(modulus)?;
        let limit = modulus.get();
        let mut len = 0_u64;
        for (word_index, output) in set.words_mut().iter_mut().enumerate() {
            let base = word_index as u64 * 64;
            let end = base + (limit - base).min(64);
            let mut word = 0_u64;
            for residue in base..end {
                if predicate(residue) {
                    word |= 1_u64 << (residue - base);
                    len += 1;
                }
            }
            *output = word;
        }
        set.len = len;
        set.assert_invariants();
        Ok(set)
    }

    /// Inserts one canonical residue and returns whether the set changed.
    pub fn insert(&mut self, residue: u64) -> Result<bool, ResidueError> {
        if residue >= self.modulus.get() {
            return Err(ResidueError::OutOfRange);
        }
        let word_index = (residue / 64) as usize;
        let mask = 1_u64 << (residue % 64);
        let word = &mut self.words_mut()[word_index];
        let changed = *word & mask == 0;
        if changed {
            *word |= mask;
            self.len += 1;
        }
        self.assert_invariants();
        Ok(changed)
    }

    /// Removes one canonical residue and returns whether the set changed.
    pub fn remove(&mut self, residue: u64) -> Result<bool, ResidueError> {
        if residue >= self.modulus.get() {
            return Err(ResidueError::OutOfRange);
        }
        let word_index = (residue / 64) as usize;
        let mask = 1_u64 << (residue % 64);
        let word = &mut self.words_mut()[word_index];
        let changed = *word & mask != 0;
        if changed {
            *word &= !mask;
            self.len -= 1;
        }
        self.assert_invariants();
        Ok(changed)
    }

    /// Returns this set's modulus.
    #[inline]
    #[must_use]
    pub const fn modulus(&self) -> Modulus {
        self.modulus
    }

    /// Returns the exact cached cardinality.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> u64 {
        self.len
    }

    /// Returns whether the set has no elements.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns whether the set contains every residue.
    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len == self.modulus.get()
    }

    /// Tests membership. Out-of-range values are not members.
    #[inline]
    #[must_use]
    pub fn contains(&self, residue: u64) -> bool {
        if residue >= self.modulus.get() {
            return false;
        }
        self.words()[(residue / 64) as usize] & (1_u64 << (residue % 64)) != 0
    }

    /// Iterates set residues in increasing order without allocation.
    #[inline]
    pub fn iter(&self) -> ResidueIter<'_> {
        ResidueIter::new(self.words(), self.len)
    }

    /// Returns the materialized intersection in one allocation and one scan.
    pub fn intersection(&self, other: &Self) -> Result<Self, ResidueError> {
        self.binary_result(other, |left, right| left & right)
    }

    /// Returns the materialized union in one allocation and one scan.
    pub fn union(&self, other: &Self) -> Result<Self, ResidueError> {
        self.binary_result(other, |left, right| left | right)
    }

    /// Returns `self` minus `other` in one allocation and one scan.
    pub fn difference(&self, other: &Self) -> Result<Self, ResidueError> {
        self.binary_result(other, |left, right| left & !right)
    }

    /// Returns the complement. Allocation failure follows Rust's normal OOM
    /// behavior because this intentionally infallible API cannot report it.
    #[must_use]
    pub fn complement(&self) -> Self {
        let mut result = Self::new_zeroed(self.modulus)
            .unwrap_or_else(|_| panic!("failed to allocate residue-set complement"));
        for (output, input) in result.words_mut().iter_mut().zip(self.words()) {
            *output = !input;
        }
        result.mask_tail();
        result.len = self.modulus.get() - self.len;
        result.assert_invariants();
        result
    }

    /// Intersects this set in place without allocation.
    pub fn intersect_assign(&mut self, other: &Self) -> Result<(), ResidueError> {
        self.ensure_same_modulus(other)?;
        if self.is_empty() || other.is_full() {
            return Ok(());
        }
        if other.is_empty() {
            self.clear();
            return Ok(());
        }
        self.len = bitops::and_assign_count(self.words_mut(), other.words());
        self.assert_invariants();
        Ok(())
    }

    /// Unites this set in place without allocation.
    pub fn union_assign(&mut self, other: &Self) -> Result<(), ResidueError> {
        self.ensure_same_modulus(other)?;
        if self.is_full() || other.is_empty() {
            return Ok(());
        }
        if other.is_full() {
            self.fill_full();
            return Ok(());
        }
        self.len = bitops::or_assign_count(self.words_mut(), other.words());
        self.assert_invariants();
        Ok(())
    }

    /// Removes another set in place without allocation.
    pub fn difference_assign(&mut self, other: &Self) -> Result<(), ResidueError> {
        self.ensure_same_modulus(other)?;
        if self.is_empty() || other.is_empty() {
            return Ok(());
        }
        if other.is_full() {
            self.clear();
            return Ok(());
        }
        self.len = bitops::and_not_assign_count(self.words_mut(), other.words());
        self.assert_invariants();
        Ok(())
    }

    /// Complements this set in place without allocation or a popcount scan.
    pub fn complement_assign(&mut self) {
        let old_len = self.len;
        for word in self.words_mut() {
            *word = !*word;
        }
        self.mask_tail();
        self.len = self.modulus.get() - old_len;
        self.assert_invariants();
    }

    /// Counts an intersection without materializing it.
    pub fn intersection_count(&self, other: &Self) -> Result<u64, ResidueError> {
        self.ensure_same_modulus(other)?;
        if self.is_empty() || other.is_empty() {
            return Ok(0);
        }
        if self.is_full() {
            return Ok(other.len);
        }
        if other.is_full() {
            return Ok(self.len);
        }
        Ok(bitops::intersection_count(self.words(), other.words()))
    }

    /// Tests whether the intersection is nonempty without materializing it.
    pub fn intersects(&self, other: &Self) -> Result<bool, ResidueError> {
        self.ensure_same_modulus(other)?;
        if self.is_empty() || other.is_empty() {
            return Ok(false);
        }
        if self.is_full() || other.is_full() {
            return Ok(true);
        }
        Ok(bitops::intersects(self.words(), other.words()))
    }

    /// Tests subset inclusion without materializing a temporary set.
    pub fn is_subset_of(&self, other: &Self) -> Result<bool, ResidueError> {
        self.ensure_same_modulus(other)?;
        if self.len > other.len {
            return Ok(false);
        }
        if self.is_empty() || other.is_full() {
            return Ok(true);
        }
        Ok(bitops::is_subset(self.words(), other.words()))
    }

    fn new_zeroed(modulus: Modulus) -> Result<Self, ResidueError> {
        let count = word_count(modulus)?;
        let storage = if count <= INLINE_WORDS {
            ResidueStorage::Inline([0; INLINE_WORDS])
        } else {
            let mut words = Vec::new();
            words
                .try_reserve_exact(count)
                .map_err(|_| ResidueError::AllocationFailed)?;
            words.resize(count, 0);
            ResidueStorage::Heap(words)
        };
        let set = Self {
            modulus,
            len: 0,
            storage,
        };
        set.assert_invariants();
        Ok(set)
    }

    fn binary_result(
        &self,
        other: &Self,
        operation: impl Fn(u64, u64) -> u64,
    ) -> Result<Self, ResidueError> {
        self.ensure_same_modulus(other)?;
        let mut result = Self::new_zeroed(self.modulus)?;
        let mut len = 0_u64;
        for ((output, left), right) in result
            .words_mut()
            .iter_mut()
            .zip(self.words())
            .zip(other.words())
        {
            let word = operation(*left, *right);
            *output = word;
            len += u64::from(word.count_ones());
        }
        result.len = len;
        result.assert_invariants();
        Ok(result)
    }

    #[inline]
    fn words(&self) -> &[u64] {
        let count = word_count_u64(self.modulus) as usize;
        match &self.storage {
            ResidueStorage::Inline(words) => &words[..count],
            ResidueStorage::Heap(words) => words,
        }
    }

    #[inline]
    fn words_mut(&mut self) -> &mut [u64] {
        let count = word_count_u64(self.modulus) as usize;
        match &mut self.storage {
            ResidueStorage::Inline(words) => &mut words[..count],
            ResidueStorage::Heap(words) => words,
        }
    }

    #[inline]
    fn ensure_same_modulus(&self, other: &Self) -> Result<(), ResidueError> {
        if self.modulus == other.modulus {
            Ok(())
        } else {
            Err(ResidueError::ModulusMismatch)
        }
    }

    fn fill_full(&mut self) {
        self.words_mut().fill(u64::MAX);
        self.mask_tail();
        self.len = self.modulus.get();
        self.assert_invariants();
    }

    fn clear(&mut self) {
        self.words_mut().fill(0);
        self.len = 0;
        self.assert_invariants();
    }

    fn mask_tail(&mut self) {
        let mask = tail_mask(self.modulus);
        let last = self.words_mut().last_mut().expect("a modulus has one word");
        *last &= mask;
    }

    #[cfg(debug_assertions)]
    fn assert_invariants(&self) {
        let expected_words = word_count_u64(self.modulus) as usize;
        assert_eq!(self.words().len(), expected_words);
        match &self.storage {
            ResidueStorage::Inline(words) => {
                assert!(expected_words <= INLINE_WORDS);
                assert!(words[expected_words..].iter().all(|word| *word == 0));
            }
            ResidueStorage::Heap(words) => {
                assert!(expected_words > INLINE_WORDS);
                assert_eq!(words.len(), expected_words);
            }
        }
        let tail = *self.words().last().expect("a modulus has one word");
        assert_eq!(tail & !tail_mask(self.modulus), 0);
        let actual: u64 = self
            .words()
            .iter()
            .map(|word| u64::from(word.count_ones()))
            .sum();
        assert_eq!(self.len, actual);
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    fn assert_invariants(&self) {}
}

/// Allocation-free iterator over set residues in increasing order.
#[derive(Clone, Debug)]
pub struct ResidueIter<'a> {
    words: &'a [u64],
    word_index: usize,
    current: u64,
    remaining: u64,
}

impl<'a> ResidueIter<'a> {
    fn new(words: &'a [u64], remaining: u64) -> Self {
        Self {
            words,
            word_index: 0,
            current: words.first().copied().unwrap_or(0),
            remaining,
        }
    }
}

impl Iterator for ResidueIter<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current != 0 {
                let bit = u64::from(self.current.trailing_zeros());
                self.current &= self.current - 1;
                self.remaining -= 1;
                return Some(self.word_index as u64 * 64 + bit);
            }
            self.word_index += 1;
            self.current = *self.words.get(self.word_index)?;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = usize::try_from(self.remaining).unwrap_or(usize::MAX);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ResidueIter<'_> {}
impl FusedIterator for ResidueIter<'_> {}

#[inline]
const fn word_count_u64(modulus: Modulus) -> u64 {
    (modulus.get() - 1) / 64 + 1
}

#[inline]
fn word_count(modulus: Modulus) -> Result<usize, ResidueError> {
    usize::try_from(word_count_u64(modulus)).map_err(|_| ResidueError::AllocationFailed)
}

#[inline]
const fn tail_mask(modulus: Modulus) -> u64 {
    let used = modulus.get() % 64;
    if used == 0 {
        u64::MAX
    } else {
        (1_u64 << used) - 1
    }
}
