/// Represents the state of a world
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WorldState {
    Stopped,
    Running,
    RunningFast,
    Success,
    Defeat,
}

/// Represents ways ticking the world can change the state of the world
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WorldStateChange {
    Win,
    Lose,
    NoChange,
}
