pub mod vec_deque_bools;

use std::{collections::VecDeque, ops::ControlFlow};

pub trait PostSystem: Clone {
    /// Initialize the system from a compressed representation of an initial string.
    fn new_decompressed(compressed: &[bool]) -> Self;

    /// Convert the system to a canonical list form.
    fn as_list(&self) -> VecDeque<bool>;

    /// Evolve the system by one step, returning [`ControlFlow::Break`] if the system halts.
    fn evolve(&mut self) -> ControlFlow<()>;
}

#[cfg(test)]
pub(crate) mod tests {
    use std::ops::ControlFlow;

    use crate::PostSystem;

    #[macro_export]
    macro_rules! tests_for_system {
        ($system:ty) => {
            #[test]
            fn decompresses() {
                $crate::tests::decompresses::<$system>();
            }

            #[test]
            fn evolves() {
                $crate::tests::evolves::<$system>();
            }
        };
    }

    pub(crate) fn decompresses<S: PostSystem>() {
        let system = S::new_decompressed(&[true]);
        assert_eq!(system.as_list().make_contiguous(), [true, false, false]);

        let system = S::new_decompressed(&[false]);
        assert_eq!(system.as_list().make_contiguous(), [false, false, false]);

        let system = S::new_decompressed(&[true, false, true, true]);
        assert_eq!(
            system.as_list().make_contiguous(),
            [true, false, false, false, false, false, true, false, false, true, false, false]
        );
    }

    pub(crate) fn evolves<S: PostSystem>() {
        let mut system = S::new_decompressed(&[true]);

        assert_eq!(system.evolve(), ControlFlow::Continue(()));
        assert_eq!(
            system.as_list().make_contiguous(),
            [true, true, false, true]
        );

        assert_eq!(system.evolve(), ControlFlow::Continue(()));
        assert_eq!(
            system.as_list().make_contiguous(),
            [true, true, true, false, true]
        );

        assert_eq!(system.evolve(), ControlFlow::Continue(()));
        assert_eq!(
            system.as_list().make_contiguous(),
            [false, true, true, true, false, true]
        );

        assert_eq!(system.evolve(), ControlFlow::Continue(()));
        assert_eq!(
            system.as_list().make_contiguous(),
            [true, false, true, false, false]
        );

        assert_eq!(system.evolve(), ControlFlow::Continue(()));
        assert_eq!(
            system.as_list().make_contiguous(),
            [false, false, true, true, false, true]
        );

        assert_eq!(system.evolve(), ControlFlow::Continue(()));
        assert_eq!(
            system.as_list().make_contiguous(),
            [true, false, true, false, false]
        );
    }
}
