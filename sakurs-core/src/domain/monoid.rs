//! Core monoid trait and operations for Delta-Stack algorithm
//!
//! This module defines the mathematical foundation that enables
//! parallelization of sentence boundary detection through monoid algebra.

use smallvec::SmallVec;

/// A mathematical monoid structure that enables parallel computation
///
/// A monoid is an algebraic structure with an associative binary operation
/// and an identity element. This enables parallel computation because:
/// - Associativity: (a ⊕ b) ⊕ c = a ⊕ (b ⊕ c)
/// - Identity: a ⊕ identity = identity ⊕ a = a
///
/// The Delta-Stack algorithm leverages these properties to split text
/// into chunks, process them independently, and combine results in any order.
pub trait Monoid: Clone + Send + Sync {
    /// Returns the identity element of the monoid
    ///
    /// The identity element must satisfy: a.combine(Self::identity()) == a
    /// for all values a of this type.
    fn identity() -> Self;

    /// Combines two elements of the monoid
    ///
    /// This operation must be associative: a.combine(b.combine(c)) == a.combine(b).combine(c)
    /// for all values a, b, c of this type.
    fn combine(&self, other: &Self) -> Self;
}

/// Extension trait for monoids that can be reduced from collections
pub trait MonoidReduce: Monoid {
    /// Reduces a collection of monoid elements to a single result
    ///
    /// Uses tree reduction to maintain O(log n) depth even with many elements.
    fn reduce<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
        I::IntoIter: ExactSizeIterator,
    {
        // Use SmallVec to avoid heap allocation for small collections
        let mut items: SmallVec<[Self; 16]> = iter.into_iter().collect();

        if items.is_empty() {
            return Self::identity();
        }

        // Tree reduction: combine pairs until only one remains
        while items.len() > 1 {
            let mut next_level = SmallVec::<[Self; 16]>::new();
            let mut i = 0;

            while i < items.len() {
                if i + 1 < items.len() {
                    next_level.push(items[i].combine(&items[i + 1]));
                    i += 2;
                } else {
                    next_level.push(items[i].clone());
                    i += 1;
                }
            }

            items = next_level;
        }

        items.into_iter().next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test monoid for verification
    #[derive(Debug, Clone, PartialEq)]
    struct TestMonoid(i32);

    impl Monoid for TestMonoid {
        fn identity() -> Self {
            TestMonoid(0)
        }

        fn combine(&self, other: &Self) -> Self {
            TestMonoid(self.0 + other.0)
        }
    }

    impl MonoidReduce for TestMonoid {}

    #[test]
    fn test_identity_property() {
        let a = TestMonoid(42);
        let id = TestMonoid::identity();

        assert_eq!(a.combine(&id), a);
        assert_eq!(id.combine(&a), a);
    }

    #[test]
    fn test_associativity() {
        let a = TestMonoid(1);
        let b = TestMonoid(2);
        let c = TestMonoid(3);

        let left = a.combine(&b).combine(&c);
        let right = a.combine(&b.combine(&c));

        assert_eq!(left, right);
    }

    #[test]
    fn test_reduce_empty() {
        let empty: Vec<TestMonoid> = vec![];
        let result = TestMonoid::reduce(empty);
        assert_eq!(result, TestMonoid::identity());
    }

    #[test]
    fn test_reduce_single() {
        let single = vec![TestMonoid(42)];
        let result = TestMonoid::reduce(single);
        assert_eq!(result, TestMonoid(42));
    }

    #[test]
    fn test_reduce_multiple() {
        let values = vec![TestMonoid(1), TestMonoid(2), TestMonoid(3), TestMonoid(4)];
        let result = TestMonoid::reduce(values);
        assert_eq!(result, TestMonoid(10));
    }
}
