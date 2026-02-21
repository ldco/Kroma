#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionState {
    Draft,
    Queued,
    Running,
    Done,
    Failed,
    Canceled,
}

impl InstructionState {
    pub fn can_transition_to(self, next: Self) -> bool {
        use InstructionState::{Canceled, Done, Draft, Failed, Queued, Running};

        matches!(
            (self, next),
            (Draft, Queued)
                | (Draft, Canceled)
                | (Queued, Running)
                | (Queued, Canceled)
                | (Running, Done)
                | (Running, Failed)
                | (Running, Canceled)
        )
    }
}
