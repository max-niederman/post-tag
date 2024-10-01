use std::{collections::VecDeque, ops::ControlFlow};

use crate::PostSystem;

#[derive(Debug, Clone)]
pub struct VecDequeBools(VecDeque<bool>);

impl PostSystem for VecDequeBools {
    fn new_decompressed(compressed: &[bool]) -> Self {
        Self(compressed.iter().flat_map(|&b| [b, false, false]).collect())
    }

    fn length(&self) -> usize {
        self.0.len()
    }

    fn as_list(&self) -> VecDeque<bool> {
        self.0.clone()
    }

    fn evolve(&mut self) -> ControlFlow<()> {
        let first = pop_front_or_break(&mut self.0)?;
        pop_front_or_break(&mut self.0)?;
        pop_front_or_break(&mut self.0)?;

        self.0.extend::<&[bool]>(match first {
            false => &[false, false],
            true => &[true, true, false, true],
        });

        ControlFlow::Continue(())
    }
}

fn pop_front_or_break<T>(deque: &mut VecDeque<T>) -> ControlFlow<(), T> {
    match deque.pop_front() {
        Some(x) => ControlFlow::Continue(x),
        None => ControlFlow::Break(()),
    }
}

#[cfg(test)]
mod tests {
    crate::tests_for_system!(super::VecDequeBools);
}
