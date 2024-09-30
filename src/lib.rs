pub mod vec_deque_bools;

use std::{collections::VecDeque, ops::ControlFlow};

pub trait PostSystem: Clone {
    /// Initialize the system from a compressed representation of an initial string.
    fn new_decompressed(compressed: &[bool]) -> Self;

    /// Convert the system to a canonical list form.
    fn as_list(&self) -> VecDeque<bool>;

    /// Evolve the system by one step, returning [`ControlFlow::Break`] if the system halts.
    fn evolve(&mut self) -> ControlFlow<()>;

    const PREFERRED_TIMESTEP: u8 = 1;

    /// Evolve the system by `n` steps.
    ///
    /// If the system halts, returns `Break(n)`, where `n` is the number of steps taken before halting.
    fn evolve_multi(&mut self, n: usize) -> ControlFlow<usize> {
        let (q, r) = (
            n / Self::PREFERRED_TIMESTEP as usize,
            n % Self::PREFERRED_TIMESTEP as usize,
        );

        for i in 0..q {
            match self.evolve_preferred() {
                ControlFlow::Break(j) => {
                    return ControlFlow::Break(i * Self::PREFERRED_TIMESTEP as usize + j as usize)
                }
                ControlFlow::Continue(()) => {}
            }
        }

        for j in 1..=r {
            match self.evolve() {
                ControlFlow::Break(()) => return ControlFlow::Break(q * Self::PREFERRED_TIMESTEP as usize + j),
                ControlFlow::Continue(()) => {}
            }
        }

        ControlFlow::Continue(())
    }

    /// Evolve the system by [`Self::PREFFERED_TIMESTEP`] steps.
    ///
    /// If the system halts, returns `Break(n)`, where `n` is the number of steps taken before halting.
    fn evolve_preferred(&mut self) -> ControlFlow<u8> {
        for i in 1..=Self::PREFERRED_TIMESTEP {
            match self.evolve() {
                ControlFlow::Break(()) => return ControlFlow::Break(i),
                ControlFlow::Continue(()) => {}
            }
        }

        ControlFlow::Continue(())
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
