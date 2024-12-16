use std::env;

/// All game configuration that can be done via env variables
#[derive(Clone)]
pub struct GameConfig {
    pub grid_radius: u8,
    pub front_end_url: String,
    pub use_benchmark_data: bool,
}

impl GameConfig {
    pub fn read_config_from_env() -> Self {
        let front_end_url = match env::var("FRONTEND_URL") {
            Ok(value) => value,
            Err(_) => "http://localhost:5173".to_string(),
        };

        let grid_radius: u8 = match env::var("GRID_RADIUS") {
            Ok(value) => value
                .parse()
                .expect("Failed to parse GRID_RADIUS. Expected a valid u8"),
            Err(_) => 80,
        };

        let use_benchmark_data: bool = match env::var("USE_BENCHMARK_DATA") {
            Ok(value) => value
                .parse()
                .expect("Failed to parse USE_BENCHMARK_DATA. Expected a boolean"),
            Err(_) => false,
        };

        Self {
            front_end_url,
            grid_radius,
            use_benchmark_data,
        }
    }
}
