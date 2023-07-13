use std::{
    fs::read_dir,                   // To find all thermal_zones
    fs::{read_to_string, DirEntry}, // To read the temperature of a thermal_zone
    thread::sleep,                  // For delay between measurements, reducing resource usage
    time::{Duration, Instant},      // For measuring performance
};

// Allows to perform multiple readings at once
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

// Performs a single reading
fn read_temperature(path: String, index: usize) -> String {
    // Read the temperature from the file, temperature will be 0 if it fails
    let temperature = read_to_string(path).unwrap_or_default();

    // Remove any whitespace at the end (or beginning)
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

        // Clear the screen
        clearscreen.clear().unwrap();

        // Read all temperatures, all temperature readings will end with a new line
        let temperatures = thermal_zones
            .clone()
            .into_par_iter()
            .map(|(path, id)| read_temperature(path, id))
            .collect::<String>();

        // Print the readings and performance
        println!("{temperatures}{}", start.elapsed().as_secs_f64());

        // Sleep for a second to save resources.
        // This shouldn't cause problems as temperatures shouldn't change that quickly.
        sleep(Duration::from_secs_f64(1.0));
    }
}
