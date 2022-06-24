use world_macros::puzzle;

// Given a the first map from Chu Chu Rocket, successfully compiles
fn main() {
    // The first level of OG ChuChu rocket
    let _map = puzzle!("Where to go?", "Sega",
    "┌───────────────────────────────────────────────────────────┐"
    "│     R         R         R         R         R         R   │"
    "├────┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼────┤"
    "│M>   M>   M>   M>   M>                                     │"
    "├────┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    │"
    "│                                                           │"
    "│    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼────┤"
    "│M>   M>   M>   M>   M>                                     │"
    "├────┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    │"
    "│                                A^ M<   M<   M<   M<   M<  │"
    "│    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼────┤"
    "│M>   M>   M>   M>   M>                                     │"
    "├────┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    │"
    "│                                   M<   M<   M<   M<   M<  │"
    "│    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼────┤"
    "│M>   M>   M>   M>   M>                                     │"
    "├────┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    ┼    │"
    "│                                   M<   M<   M<   M<   M<  │"
    "└───────────────────────────────────────────────────────────┘");

    // TODO: Add assertions on contents
}
