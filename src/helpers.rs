//! Fundamental helper functions

use std::{collections::HashSet, hash::Hash};

/// Removes multiple elements from a vector, given a collection of indices to remove.
pub fn remove_multiple<T>(vector: &mut Vec<T>, to_remove: &Vec<usize>) {
    // As items are removed, their indices will change, so we have to keep track of the new indices.
    let mut to_remove = to_remove.to_vec();

    for i_tr in 0..to_remove.len() {
        let index_last = vector.len() - 1;
        let tr = to_remove[i_tr];
        assert!(tr <= index_last);
        vector.swap_remove(tr);

        // Find and swap any corresponding to_remove entry*
        let mut to_swap: Option<usize> = None;
        for j_tr in 0..to_remove.len() {
            // *i.e. any index pointing to the previously last item
            if to_remove[j_tr] == index_last {
                to_swap = Some(j_tr);
                break;
            }
        }
        if let Some(to_swap) = to_swap {
            to_remove[to_swap] = tr;
        }
    }
}

/// Tests whether the given vectors contain the same items, regardless of order
/// from: https://stackoverflow.com/a/42748484
//pub fn set_eq<T>
pub fn set_eq<T>(a: &[T], b: &[T]) -> bool
where
    T: Eq + Hash,
{
    let a: HashSet<_> = a.iter().collect();
    let b: HashSet<_> = b.iter().collect();

    a == b
}

#[cfg(test)]
mod tests {
    use super::{remove_multiple, set_eq};

    #[test]
    fn remove_multiple_0_3() {
        let mut vec = vec!["a", "b", "c"];
        let to_remove = vec![];

        remove_multiple(&mut vec, &to_remove);

        assert_eq!(vec, vec!["a", "b", "c"]);
    }

    #[test]
    fn remove_multiple_1_1() {
        // Arrange
        let mut vec = vec!["a"];
        let to_remove = vec![0];

        // Act
        remove_multiple(&mut vec, &to_remove);

        // Assert
        assert_eq!(vec, Vec::<&str>::new());
    }

    #[test]
    fn remove_multiple_1_3() {
        // Arrange
        let mut vec = vec!["a", "b", "c"];
        let to_remove = vec![0];

        // Act
        remove_multiple(&mut vec, &to_remove);

        // Assert
        // Order swapped due to implementation detail - SHOULDDO: order independent assertion
        assert_eq!(vec, vec!["c", "b"]);
    }

    #[test]
    fn remove_multiple_2_3() {
        // Arrange
        let mut vec = vec!["a", "b", "c"];
        let to_remove = vec![0, 2];

        // Act
        remove_multiple(&mut vec, &to_remove);

        // Assert
        assert_eq!(vec, vec!["b"]);
    }

    #[test]
    fn remove_multiple_2_3_rev() {
        // Arrange
        let mut vec = vec!["a", "b", "c"];
        let to_remove = vec![2, 0];

        // Act
        remove_multiple(&mut vec, &to_remove);

        // Assert
        assert_eq!(vec, vec!["b"]);
    }

    #[test]
    fn remove_multiple_3_4() {
        // Arrange
        let mut vec = vec!["a", "b", "c", "d"];
        let to_remove = vec![1, 3, 2];

        // Act
        remove_multiple(&mut vec, &to_remove);

        // Assert
        assert_eq!(vec, vec!["a"]);
    }

    #[test]
    #[should_panic]
    fn remove_multiple_out_of_bounds() {
        // Arrange
        let mut vec = vec!["a", "b", "c", "d"];
        let to_remove = vec![4];

        // Act
        remove_multiple(&mut vec, &to_remove);
    }

    #[test]
    fn set_eq_test() {
        assert!(set_eq(
            &vec!["foo", "bar", "baz", "beh"],
            &vec!["beh", "foo", "baz", "bar"]
        ));
        assert!(!set_eq(
            &vec!["beh", "foo", "baz", "bar"],
            &vec!["beh", "foo", "baz", "baz"]
        ));
    }
}
