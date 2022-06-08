use common::input::InputState;

use crate::World;

/// A tickable thing
trait TickableState {
    fn tick(&mut self, input: &InputState) -> Option<AppState>;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct IntroState {
    frame: u32,
    transition_started: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct MenuState {
    map_index: u16,
    map_count: u16,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct GameState {}

// Top level states the game can be in
//TODO: The map should be a parameter of the game enum!
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AppState {
    Intro(IntroState),
    Menu(MenuState),
    Game(GameState),
}

impl TickableState for AppState {
    fn tick(&mut self, input: &InputState) -> Option<AppState> {
        match self {
            AppState::Intro(state) => state.tick(input),
            AppState::Menu(state) => state.tick(input),
            AppState::Game(state) => state.tick(input),
        }
    }
}

impl TickableState for IntroState {
    fn tick(&mut self, input: &InputState) -> Option<AppState> {
        self.frame += 1;

        // Goto menu if any button pressed
        // Goto menu automatically after a couple of seconds
        if !self.transition_started
            && (input.btn_start.pressed
                || input.btn_a.pressed
                || input.btn_b.pressed
                || self.frame == 120)
        {
            self.transition_started = true;

            Some(AppState::Menu(MenuState {
                map_count: 10,
                map_index: 0,
            }))
        } else {
            None
        }
    }
}

impl TickableState for MenuState {
    fn tick(&mut self, input: &InputState) -> Option<AppState> {
        if input.js_up.pressed {
            self.map_index = match self.map_index {
                0 => 10,
                other => other - 1
            };
        }

        if input.js_down.pressed {
            self.map_index = match self.map_index {
                10 => 0,
                other => other + 1
            };
        }

        None
    }
}

impl TickableState for GameState {
    fn tick(&mut self, input: &InputState) -> Option<AppState> {
        None
    }
}

/// The top level state machine
pub struct StateMachine {
    /// The current game state
    state: AppState,

    /// The currently loaded/selected world
    /// This has to be stored outside the state, as it's not copiable
    world: World,

    // The target game state
    target_state: AppState,

    // The number of frames left on a transition between states
    transition_timer: u16,
}

impl StateMachine {
    pub fn new() -> StateMachine {
        let initial_state = AppState::Intro(IntroState {
            frame: 0,
            transition_started: false,
        });

        StateMachine {
            state: initial_state,
            world: World::new(),
            target_state: initial_state,
            transition_timer: 0,
        }
    }

    pub fn tick(&mut self, input: &InputState) {
        // Handle transition between states
        if self.state != self.target_state {
            if self.transition_timer > 0 {
                self.transition_timer -= 1;
            } else {
                self.state = self.target_state;
            }
        }

        // Tick the current state, and handle requests to the next state
        match self.state.tick(input) {
            Some(target_state) => {
                self.target_state = target_state;
                self.transition_timer = 45;
            }
            None => {}
        }
    }
}
