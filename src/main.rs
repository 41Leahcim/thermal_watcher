use std::time::{Duration, Instant};

use smol::stream::StreamExt;

const DELAY: Duration = Duration::from_secs(1);

// Performs a single reading
async fn read_temperature(path: String, index: usize) -> String {
    // Read the temperature from the file, temperature will be 0 if it fails
    let temperature = smol::fs::read_to_string(path).await.unwrap_or_default();

    // Remove any whitespace at the beginning or end
    let temperature = temperature.trim();

    // Create a formatted string containing the thermal_zone index and the current temperature
    format!("thermal_zone{index}: {temperature}\n")
}

// Returns the thermal_zone path and id, if the DirEntry is a thermal_zone
async fn thermal_zone_filter(
    entry: Result<smol::fs::DirEntry, std::io::Error>,
) -> Option<(String, usize)> {
    // Ignore entries that can't be read
    let Ok(entry) = entry else { return None };

    // Ignore entries without file type
    let Ok(file_type) = entry.file_type().await else {
        return None;
    };

    // Ignore entries with file names that can't be converted to &str
    let file_name = entry.file_name();
    let Some(file_name) = file_name.to_str() else {
        return None;
    };

    // Ignore paths that can't be converted to &str
    let path = entry.path();
    let Some(path) = path.to_str() else {
        return None;
    };

    // A thermal_zone could be a symlink or directory.
    // It's filename always start with thermal_zone.
    if !file_type.is_file() && file_name.starts_with("thermal_zone") {
        // Take the id at the end of the filename
        let id = file_name
            .chars()
            .skip("thermal_zone".len())
            .collect::<String>()
            .trim()
            .parse::<usize>();

        // Ignore the thermal_zone, if it doesn't have an id
        let Ok(id) = id else {
            return None;
        };

        // Return the thermal_zone path and id
        Some((path.to_string() + "/temp", id))
    } else {
        // Return None as this isn't a thermal_zone
        None
    }
}

fn find_thermal_zones() -> Vec<(String, usize)> {
    smol::block_on(async {
        smol::fs::read_dir("/sys/class/thermal/") // Read the thermal directory
            .await // Wait for the directory reading iterator to be created
            // Panic (exit with error) if the directory doesn't exist or the user doesn't have
            .unwrap()
            // filter the directory, only keeping the thermal zones
            .map(|entry| smol::spawn(thermal_zone_filter(entry)))
            .collect::<Vec<_>>()
            .await // Wait for the directory to be read
    })
    .into_iter()
    // Wait for the directory to be filtered, only keeping the thermal zone with the zone id
    .filter_map(smol::block_on)
    .collect::<Vec<(String, usize)>>() // Collect all thermal_zones with their id in a vec
}

fn read_temperatures(thermal_zones: &[(String, usize)]) -> Vec<smol::Task<String>> {
    thermal_zones
        .iter()
        .map(|(path, id)| smol::spawn(read_temperature(path.clone(), *id)))
        .collect::<Vec<_>>()
}

fn main() {
    let mut thermal_zones = find_thermal_zones();

    // Sort thermal_zones on id
    thermal_zones.sort_by(|a, b| a.1.cmp(&b.1));

    // Create a ClearScreen object to clear the CLI more efficiently
    let clearscreen = clearscreen::ClearScreen::default();

    // Log the temperatures every second
    loop {
        // Start measuring performance
        let start = Instant::now();

        // Start tasks for reading the thermal zones
        let tasks = read_temperatures(&thermal_zones);

        // Clear the screen
        clearscreen.clear().unwrap();

        // Wait until all thermal zones have been read, add the results to a string.
        let temperatures = tasks.into_iter().map(smol::block_on).collect::<String>();

        // Print the readings and performance
        println!("{temperatures}{}", start.elapsed().as_secs_f64());

        std::thread::sleep(DELAY);
    }
}
