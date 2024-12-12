const DATA_FILE: &str = "game_data.json";

fn load_data_from_file(radius: i32) -> GameData {
    if let Ok(mut file) = File::open(DATA_FILE) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(data) = serde_json::from_str::<GameData>(&contents) {
                return data;
            }
        }
    }

    GameData::new(radius)
}

async fn periodic_save(data: Data<RwLock<GameData>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await; // Save every 30 seconds
        if let Ok(store) = data.write() {
            if let Ok(serialized) = serde_json::to_string(&*store) {
                if let Ok(mut file) = File::create(DATA_FILE) {
                    let _ = file.write_all(serialized.as_bytes());
                }
            }
        }
    }
}
