use std::env;

/// All game configuration that can be done via env variables
#[derive(Clone)]
pub struct GameConfig {
    pub front_end_url: String,
    pub grid_batch_div: u8,
    pub grid_radius: u32,
    pub locust_url: String,
    pub redis_url: String,
    pub use_benchmark_data: bool,
    pub with_redis_tests: bool,
}

impl GameConfig {
    pub fn read_config_from_env() -> Self {
        let front_end_url = match env::var("FRONTEND_URL") {
            Ok(value) => value,
            Err(_) => "http://localhost:5173".to_string(),
        };

        let locust_url = match env::var("LOCUST_URL") {
            Ok(value) => value,
            Err(_) => "http://localhost:8081".to_string(),
        };

        let grid_radius: u32 = match env::var("GRID_RADIUS") {
            Ok(value) => value
                .parse()
                .expect("Failed to parse GRID_RADIUS. Expected a valid u32"),
            Err(_) => 80,
        };

        let grid_batch_div: u8 = match env::var("GRID_BATCH_DIV") {
            Ok(value) => value
                .parse()
                .expect("Failed to parse GRID_BATCH_DIV. Expected a valid u8"),
            Err(_) => 8,
        };

        let use_benchmark_data: bool = match env::var("USE_BENCHMARK_DATA") {
            Ok(value) => value
                .parse()
                .expect("Failed to parse USE_BENCHMARK_DATA. Expected a boolean"),
            Err(_) => false,
        };

        let with_redis_tests: bool = match env::var("WITH_REDIS_TESTS") {
            Ok(value) => value
                .parse()
                .expect("Failed to parse WITH_REDIS_TESTS. Expected a boolean"),
            Err(_) => false,
        };

        let redis_url = match env::var("REDIS_URL") {
            Ok(value) => value,
            Err(_) => "redis://127.0.0.1:6379".to_string(),
        };

        Self {
            front_end_url,
            grid_batch_div,
            grid_radius,
            locust_url,
            redis_url,
            use_benchmark_data,
            with_redis_tests,
        }
    }
}
