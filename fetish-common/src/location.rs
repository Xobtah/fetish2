use std::{
    fmt::Display,
    // hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
enum Direction {
    North = 0,
    NorthEast = 60,
    SouthEast = 120,
    South = 180,
    SouthWest = 240,
    NorthWest = 300,
}

impl Direction {
    fn f64(&self) -> f64 {
        *self as i32 as f64
    }

    fn next(&self) -> Direction {
        match self {
            Direction::North => Direction::SouthEast,
            Direction::NorthEast => Direction::South,
            Direction::SouthEast => Direction::SouthWest,
            Direction::South => Direction::NorthWest,
            Direction::SouthWest => Direction::North,
            Direction::NorthWest => Direction::NorthEast,
        }
    }

    fn iter() -> impl Iterator<Item = Direction> {
        [
            Direction::North,
            Direction::NorthEast,
            Direction::SouthEast,
            Direction::South,
            Direction::SouthWest,
            Direction::NorthWest,
        ]
        .iter()
        .copied()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct Location(pub f64, pub f64);

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.round(6).distance_to(&other.round(6)) < 10.0
    }
}

impl Location {
    pub fn new(lat: f64, lon: f64) -> Location {
        Location(lat, lon)
    }

    fn translate(&self, direction: Direction, distance: f64) -> Location {
        let earth_radius = 6_371_000.0; // meters
        let lat_rad = self.0.to_radians();
        let lon_rad = self.1.to_radians();
        let dir_rad = direction.f64().to_radians();
        let new_lat = (lat_rad + (distance / earth_radius) * dir_rad.cos()).to_degrees();
        let new_lon =
            (lon_rad + (distance / earth_radius) * dir_rad.sin() / lat_rad.cos()).to_degrees();
        Location(new_lat, new_lon)
    }

    pub fn distance_to(&self, other: &Location) -> f64 {
        let earth_radius = 6_371_000.0; // meters
        let lat1_rad = self.0.to_radians();
        let lon1_rad = self.1.to_radians();
        let lat2_rad = other.0.to_radians();
        let lon2_rad = other.1.to_radians();
        let delta_lat = lat2_rad - lat1_rad;
        let delta_lon = lon2_rad - lon1_rad;
        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        earth_radius * c
    }

    pub fn round(&self, decimals: u32) -> Location {
        let factor = 10_f64.powi(decimals as i32);
        Location(
            (self.0 * factor).round() / factor,
            (self.1 * factor).round() / factor,
        )
    }

    pub fn compute_locations(&self, distance: f64, level: u32) -> Vec<Location> {
        fn compute_branch(
            origin: &Location,
            distance: f64,
            direction: Direction,
            amount: u32,
        ) -> Vec<Location> {
            (1..amount + 1)
                .map(|i| origin.translate(direction, distance * i as f64))
                .collect()
        }

        let mut locations = vec![*self];
        for l in 1..level + 1 {
            Direction::iter()
                .map(|d| (self.translate(d, distance * l as f64), d.next()))
                .for_each(|(branch_origin, d)| {
                    locations.append(&mut compute_branch(&branch_origin, distance, d, l));
                });
        }
        locations
    }
}

impl Into<tdlib::types::Location> for Location {
    fn into(self) -> tdlib::types::Location {
        tdlib::types::Location {
            latitude: self.0,
            longitude: self.1,
            horizontal_accuracy: 0.,
        }
    }
}

impl Into<tdlib::types::Location> for &Location {
    fn into(self) -> tdlib::types::Location {
        tdlib::types::Location {
            latitude: self.0,
            longitude: self.1,
            horizontal_accuracy: 0.,
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_wgs84_location() {
        let orig = Location::new(48.859270, 2.382861);
        let loc = orig.translate(Direction::North, 860.0); // NORTH
        assert_eq!(Location::new(48.867004, 2.382861), loc.round(6));
        let loc = orig.translate(Direction::NorthEast, 860.0); // NORTH EAST
        assert_eq!(Location::new(48.863137, 2.393042), loc.round(6));
        let loc = orig.translate(Direction::SouthEast, 860.0); // SOUTH EAST
        assert_eq!(Location::new(48.855403, 2.393042), loc.round(6));
        let loc = orig.translate(Direction::South, 860.0); // SOUTH
        assert_eq!(Location::new(48.851536, 2.382861), loc.round(6));
        let loc = orig.translate(Direction::SouthWest, 860.0); // SOUTH WEST
        assert_eq!(Location::new(48.855403, 2.37268), loc.round(6));
        let loc = orig.translate(Direction::NorthWest, 860.0); // NORTH WEST
        assert_eq!(Location::new(48.863137, 2.37268), loc.round(6));
    }

    #[test]
    fn test_location_distance() {
        let loc1 = Location(48.859270, 2.382861);
        let loc2 = Location(48.867004165810904, 2.382861);
        assert_eq!(loc1.distance_to(&loc2).round(), 860.0);
    }

    #[test]
    fn test_location_round() {
        let loc = Location(48.859270, 2.382861);
        assert_eq!(loc.round(2), Location(48.86, 2.38));
    }

    #[test]
    fn test_location_overlap() {
        let orig = Location::new(48.859270, 2.382861);
        let loc = orig.translate(Direction::SouthEast, 18.);
        let loc1 = loc.translate(Direction::NorthEast, 24.);
        let loc2 = loc1.translate(Direction::SouthWest, 24.);
        assert_eq!(loc.round(6), loc2.round(6));
    }

    #[test]
    fn test_compute_locations() {
        let origin = Location::new(48.859270, 2.382861);
        let distance = 860.;

        let locations = origin.compute_locations(distance, 0);
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0], origin);

        let locations = origin.compute_locations(distance, 1);
        assert_eq!(locations.len(), 7);
        assert_eq!(locations[0], origin);
        assert_eq!(
            locations[1],
            origin.translate(Direction::NorthEast, distance)
        );
        assert_eq!(
            locations[2],
            origin.translate(Direction::SouthEast, distance)
        );
        assert_eq!(locations[3], origin.translate(Direction::South, distance));
        assert_eq!(
            locations[4],
            origin.translate(Direction::SouthWest, distance)
        );
        assert_eq!(
            locations[5],
            origin.translate(Direction::NorthWest, distance)
        );
        assert_eq!(locations[6], origin.translate(Direction::North, distance));

        let locations = origin.compute_locations(distance, 2);
        assert_eq!(locations.len(), 19);
        assert_eq!(locations[0], origin);
        assert_eq!(
            locations[1],
            origin.translate(Direction::NorthEast, distance)
        );
        assert_eq!(
            locations[2],
            origin.translate(Direction::SouthEast, distance)
        );
        assert_eq!(locations[3], origin.translate(Direction::South, distance));
        assert_eq!(
            locations[4],
            origin.translate(Direction::SouthWest, distance)
        );
        assert_eq!(
            locations[5],
            origin.translate(Direction::NorthWest, distance)
        );
        assert_eq!(locations[6], origin.translate(Direction::North, distance));
        assert_eq!(
            locations[7],
            locations[1].translate(Direction::SouthEast, distance)
        );

        // let locations = compute_locations(&origin, distance, 3);
        // assert_eq!(locations.len(), 37);
        // assert_eq!(locations[0], origin);
        // assert!(locations
        //     .iter()
        //     .skip(19)
        //     .map(|l| l.distance_to(&origin) - (distance * 3.))
        //     .any(|d| d < 1. && d > -0.));
    }
}
