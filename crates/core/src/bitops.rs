#[inline]
pub(crate) fn and_assign_count(lhs: &mut [u64], rhs: &[u64]) -> u64 {
    debug_assert_eq!(lhs.len(), rhs.len());
    let mut count = 0;
    for (left, right) in lhs.iter_mut().zip(rhs) {
        *left &= *right;
        count += u64::from(left.count_ones());
    }
    count
}

#[inline]
pub(crate) fn or_assign_count(lhs: &mut [u64], rhs: &[u64]) -> u64 {
    debug_assert_eq!(lhs.len(), rhs.len());
    let mut count = 0;
    for (left, right) in lhs.iter_mut().zip(rhs) {
        *left |= *right;
        count += u64::from(left.count_ones());
    }
    count
}

#[inline]
pub(crate) fn and_not_assign_count(lhs: &mut [u64], rhs: &[u64]) -> u64 {
    debug_assert_eq!(lhs.len(), rhs.len());
    let mut count = 0;
    for (left, right) in lhs.iter_mut().zip(rhs) {
        *left &= !*right;
        count += u64::from(left.count_ones());
    }
    count
}

#[inline]
pub(crate) fn intersection_count(lhs: &[u64], rhs: &[u64]) -> u64 {
    debug_assert_eq!(lhs.len(), rhs.len());
    lhs.iter()
        .zip(rhs)
        .map(|(left, right)| u64::from((left & right).count_ones()))
        .sum()
}

#[inline]
pub(crate) fn intersects(lhs: &[u64], rhs: &[u64]) -> bool {
    debug_assert_eq!(lhs.len(), rhs.len());
    lhs.iter().zip(rhs).any(|(left, right)| left & right != 0)
}

#[inline]
pub(crate) fn is_subset(lhs: &[u64], rhs: &[u64]) -> bool {
    debug_assert_eq!(lhs.len(), rhs.len());
    lhs.iter().zip(rhs).all(|(left, right)| left & !right == 0)
}
