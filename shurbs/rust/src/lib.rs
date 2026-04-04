pub mod api;
pub mod commands;
pub mod error;
pub mod state;


// Re-export kernel init so Flutter can call it on app start.
pub use state::init_kernel;
