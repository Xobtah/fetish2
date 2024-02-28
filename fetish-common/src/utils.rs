use dialoguer::{theme::ColorfulTheme, Input};

use crate::error::FetishResult;

pub fn ask_user(prompt: &str) -> FetishResult<String> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()?
        .trim()
        .into())
}

// pub fn translate_wgs84_location(
//     latitude: f64,
//     longitude: f64,
//     direction: f64,
//     length: f64,
// ) -> (f64, f64) {
//     let earth_radius = 6_371_000.0; // meters
//     let lat_rad = latitude.to_radians();
//     let lon_rad = longitude.to_radians();
//     let dir_rad = direction.to_radians();
//     let new_lat = (lat_rad + (length / earth_radius) * dir_rad.cos()).to_degrees();
//     let new_lon = (lon_rad + (length / earth_radius) * dir_rad.sin() / lat_rad.cos()).to_degrees();
//     (new_lat, new_lon)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_translate_wgs84_location() {
//         let (orig_lat, orig_lon) = (48.859270, 2.382861);
//         let (lat, lon) = translate_wgs84_location(orig_lat, orig_lon, 0., 860.0); // NORTH
//         assert_eq!(lat, 48.867004165810904);
//         assert_eq!(lon, 2.382861);
//         let (lat, lon) = translate_wgs84_location(orig_lat, orig_lon, 45., 860.0); // NORTH EAST
//         assert_eq!(lat, 48.86473888109171);
//         assert_eq!(lon, 2.391173496715638);
//         let (lat, lon) = translate_wgs84_location(orig_lat, orig_lon, 90., 860.0); // EAST
//         assert_eq!(lat, 48.859270);
//         assert_eq!(lon, 2.394616645592437);
//         let (lat, lon) = translate_wgs84_location(orig_lat, orig_lon, 135., 860.0); // SOUTH EAST
//         assert_eq!(lat, 48.853801118908294);
//         assert_eq!(lon, 2.391173496715638);
//         let (lat, lon) = translate_wgs84_location(orig_lat, orig_lon, 180., 860.0); // SOUTH
//         assert_eq!(lat, 48.85153583418911);
//         assert_eq!(lon, 2.382861);
//         let (lat, lon) = translate_wgs84_location(orig_lat, orig_lon, 225., 860.0); // SOUTH WEST
//         assert_eq!(lat, 48.853801118908294);
//         assert_eq!(lon, 2.3745485032843625);
//         let (lat, lon) = translate_wgs84_location(orig_lat, orig_lon, 270., 860.0); // WEST
//         assert_eq!(lat, 48.859270);
//         assert_eq!(lon, 2.3711053544075633);
//         let (lat, lon) = translate_wgs84_location(orig_lat, orig_lon, 315., 860.0); // NORTH WEST
//         assert_eq!(lat, 48.86473888109171);
//         assert_eq!(lon, 2.3745485032843625);
//     }
// }
