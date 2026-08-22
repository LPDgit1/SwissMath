use std::collections::BTreeSet;

pub fn set_from_mask(modulus: u64, mask: u64) -> BTreeSet<u64> {
    (0..modulus)
        .filter(|residue| mask & (1_u64 << residue) != 0)
        .collect()
}

pub fn intersection(left: &BTreeSet<u64>, right: &BTreeSet<u64>) -> BTreeSet<u64> {
    left.intersection(right).copied().collect()
}

pub fn union(left: &BTreeSet<u64>, right: &BTreeSet<u64>) -> BTreeSet<u64> {
    left.union(right).copied().collect()
}

pub fn difference(left: &BTreeSet<u64>, right: &BTreeSet<u64>) -> BTreeSet<u64> {
    left.difference(right).copied().collect()
}

pub fn complement(modulus: u64, set: &BTreeSet<u64>) -> BTreeSet<u64> {
    (0..modulus).filter(|value| !set.contains(value)).collect()
}
