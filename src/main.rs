use std::{
    fs::read_dir,
    fs::read_to_string,
    thread::sleep,
    time::{Duration, Instant},
};

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

fn read_temperature(path: String, index: usize) -> String {
    let temperature = read_to_string(path).unwrap_or_default();
    let temperature = temperature.trim();
    format!("thermal_zone{index}: {temperature}\n")
}

fn main() {
    let mut thermal_zones = read_dir("/sys/class/thermal/")
        .unwrap()
        .filter_map(|entry| {
            let Ok(entry) = entry else { return None };
            let Ok(file_type) = entry.file_type() else {
                return None;
            };
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                return None;
            };
            let path = entry.path();
            let Some(path) = path.to_str() else {
                return None;
            };
            if !file_type.is_file() && file_name.starts_with("thermal_zone") {
                let id = file_name
                    .chars()
                    .skip("thermal_zone".len())
                    .collect::<String>()
                    .trim()
                    .parse::<usize>();
                let Ok(id) = id else {
                    return None;
                };
                Some((path.to_string() + "/temp", id))
            } else {
                None
            }
        })
        .collect::<Vec<(String, usize)>>();
    thermal_zones.sort_by(|a, b| a.1.cmp(&b.1));
    let clearscreen = clearscreen::ClearScreen::default();
    loop {
        clearscreen.clear().unwrap();
        let start = Instant::now();
        let temperatures = thermal_zones
            .clone()
            .into_par_iter()
            .map(|(path, id)| read_temperature(path, id))
            .collect::<String>();
        println!("{temperatures}{}", start.elapsed().as_secs_f64());
        sleep(Duration::from_secs_f64(1.0));
    }
}
