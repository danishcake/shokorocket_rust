use world_macros::puzzle;

// Given empty input the test should fail to compile
fn main() {
    let _map = puzzle!("", "Author",
    "┌───────────────────────────────────────────────────────────┐"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "│                                                           │"
    "└───────────────────────────────────────────────────────────┘");
}
