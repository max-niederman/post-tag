pub mod vec_deque_bools;
pub mod bitstring;

use std::{collections::VecDeque, ops::ControlFlow};

pub trait PostSystem: Clone {
    /// Initialize the system from a compressed representation of an initial string.
    fn new_decompressed(compressed: &[bool]) -> Self;

    /// Get the length of the system.
    fn length(&self) -> usize;

    /// Convert the system to a canonical list form.
    fn as_list(&self) -> VecDeque<bool>;

    /// Evolve the system by one step, returning [`ControlFlow::Break`] if the system halts.
    fn evolve(&mut self) -> ControlFlow<()>;

    /// Evolve the system by `n` steps.
    ///
    /// If the system halts, returns `Break(n)`, where `n` is the number of steps taken before halting.
    fn evolve_multi(&mut self, n: usize) -> ControlFlow<usize> {
        let mut i = 0;
        while i < n {
            if self.length() >= 3 * Self::PREFERRED_TIMESTEP as usize {
                self.evolve_preferred();
                i += Self::PREFERRED_TIMESTEP as usize;
            } else {
                let res = self.evolve();

                i += 1;

                if let ControlFlow::Break(()) = res {
                    return ControlFlow::Break(i);
                }
            }
        }

        ControlFlow::Continue(())
    }

    /// The preferred number of steps to take when evolving the system.
    const PREFERRED_TIMESTEP: u8 = 1;

    /// Evolve the system by [`Self::PREFFERED_TIMESTEP`] steps.
    ///
    /// The result of calling this on a system with length less than `3 * Self::PREFERRED_TIMESTEP` is undefined.
    fn evolve_preferred(&mut self) {
        debug_assert!(self.length() >= 3 * Self::PREFERRED_TIMESTEP as usize);

        for _ in 0..Self::PREFERRED_TIMESTEP {
            self.evolve();
        }
    }
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
