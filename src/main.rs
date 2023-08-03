use std::{
    fs::read_dir,              // To find all thermal_zones
    fs::DirEntry,              // To read the temperature of a thermal_zone
    thread::sleep,             // For delay between measurements, reducing resource usage
    time::{Duration, Instant}, // For measuring performance
};

// Allows to perform multiple readings at once
//use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

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
fn thermal_zone_filter(entry: Result<DirEntry, std::io::Error>) -> Option<(String, usize)> {
    // Ignore entries that can't be read
    let Ok(entry) = entry else { return None };

    // Ignore entries without file type
    let Ok(file_type) = entry.file_type() else {
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

fn main() {
    // Read the /sys/class/thermal/ directory to find all thermal_zones.
    // This fails if the /sys/class/thermal directory doesn't exist,
    // like on any OS other than Linux.
    let mut thermal_zones = read_dir("/sys/class/thermal/")
        .unwrap()
        .filter_map(thermal_zone_filter) // Collect all thermal_zones with their id in a vec
        .collect::<Vec<(String, usize)>>();

    // Sort thermal_zones on id
    thermal_zones.sort_by(|a, b| a.1.cmp(&b.1));

    // Create a ClearScreen object to clear the CLI more efficiently
    let clearscreen = clearscreen::ClearScreen::default();
    loop {
        // Start measuring performance
        let start = Instant::now();

        let tasks = thermal_zones
            .iter()
            .map(|(path, id)| smol::spawn(read_temperature(path.clone(), *id)))
            .collect::<Vec<_>>();

        // Clear the screen
        clearscreen.clear().unwrap();

        // Read all temperatures, all temperature readings will end with a new line
        let temperatures = tasks.into_iter().map(smol::block_on).collect::<String>();

        // Print the readings and performance
        println!("{temperatures}{}", start.elapsed().as_secs_f64());

        // Sleep for a second to save resources.
        // This shouldn't cause problems as temperatures shouldn't change that quickly.
        sleep(Duration::from_secs_f64(1.0));
    }
}
