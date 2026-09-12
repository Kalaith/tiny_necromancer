//! Public game logic shared by the Macroquad binary and integration tests.

pub mod data;
pub mod engine;
pub mod game;
pub mod name_generator;
pub mod state;
pub mod ui;

pub use data::GameData;
pub use engine::TickReport;
pub use game::Game;
pub use state::{GamePhase, GameSession};
