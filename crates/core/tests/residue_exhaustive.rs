mod reference;

use std::collections::BTreeSet;

use swissmath_core::{Modulus, ResidueError, ResidueSet};

fn modulus(value: u64) -> Modulus {
    Modulus::new(value).unwrap()
}

fn materialize(modulus: u64, mask: u64) -> ResidueSet {
    ResidueSet::from_iter(
        self::modulus(modulus),
        (0..modulus).filter(|residue| mask & (1_u64 << residue) != 0),
    )
    .unwrap()
}

fn values(set: &ResidueSet) -> BTreeSet<u64> {
    let result: BTreeSet<_> = set.iter().collect();
    assert_eq!(set.len(), result.len() as u64);
    result
}

#[test]
fn exhaustive_all_subsets_through_modulus_seven() {
    for m in 1..=7 {
        let subset_count = 1_u64 << m;
        for left_mask in 0..subset_count {
            let left_ref = reference::set_from_mask(m, left_mask);
            let left = materialize(m, left_mask);
            assert_eq!(values(&left), left_ref);
            assert_eq!(left.is_empty(), left_ref.is_empty());
            assert_eq!(left.is_full(), left_ref.len() as u64 == m);

            assert_eq!(
                values(&left.complement()),
                reference::complement(m, &left_ref)
            );
            let mut twice = left.clone();
            twice.complement_assign();
            twice.complement_assign();
            assert_eq!(twice, left);

            for residue in 0..m {
                let mut edited = left.clone();
                let expected_insert = !left_ref.contains(&residue);
                assert_eq!(edited.insert(residue).unwrap(), expected_insert);
                assert!(!edited.insert(residue).unwrap());
                assert!(edited.contains(residue));
                assert_eq!(edited.len(), edited.iter().count() as u64);
                assert!(edited.remove(residue).unwrap());
                assert!(!edited.remove(residue).unwrap());
                assert_eq!(edited.len(), edited.iter().count() as u64);
            }

            for right_mask in 0..subset_count {
                let right_ref = reference::set_from_mask(m, right_mask);
                let right = materialize(m, right_mask);

                let intersection = reference::intersection(&left_ref, &right_ref);
                let union = reference::union(&left_ref, &right_ref);
                let difference = reference::difference(&left_ref, &right_ref);

                assert_eq!(values(&left.intersection(&right).unwrap()), intersection);
                assert_eq!(values(&left.union(&right).unwrap()), union);
                assert_eq!(values(&left.difference(&right).unwrap()), difference);
                assert_eq!(
                    left.intersection_count(&right).unwrap(),
                    intersection.len() as u64
                );
                assert_eq!(left.intersects(&right).unwrap(), !intersection.is_empty());
                assert_eq!(
                    left.is_subset_of(&right).unwrap(),
                    left_ref.is_subset(&right_ref)
                );

                let mut assigned = left.clone();
                assigned.intersect_assign(&right).unwrap();
                assert_eq!(values(&assigned), intersection);
                let mut assigned = left.clone();
                assigned.union_assign(&right).unwrap();
                assert_eq!(values(&assigned), union);
                let mut assigned = left.clone();
                assigned.difference_assign(&right).unwrap();
                assert_eq!(values(&assigned), difference);
            }
        }
    }
}

#[test]
fn rejects_invalid_residues_and_mismatched_moduli() {
    let mut set = ResidueSet::try_empty(modulus(7)).unwrap();
    assert_eq!(set.insert(7), Err(ResidueError::OutOfRange));
    assert_eq!(set.remove(7), Err(ResidueError::OutOfRange));
    assert!(!set.contains(7));
    assert_eq!(
        ResidueSet::singleton(modulus(7), 7),
        Err(ResidueError::OutOfRange)
    );
    assert_eq!(
        ResidueSet::from_iter(modulus(7), [1, 7]),
        Err(ResidueError::OutOfRange)
    );
    let other = ResidueSet::try_empty(modulus(8)).unwrap();
    assert_eq!(set.intersection(&other), Err(ResidueError::ModulusMismatch));
    assert_eq!(
        set.intersection_count(&other),
        Err(ResidueError::ModulusMismatch)
    );
}
