use csv::Writer;
use roxmltree::Document;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, str::FromStr};

#[derive(Debug, PartialEq)]
struct Coordinate {
    degrees: i32,
    minutes: u32,
    seconds: f32,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseCoordinateError;
#[derive(Debug, PartialEq, Eq)]
struct ParsePositionError;

impl FromStr for Coordinate {
    type Err = ParseCoordinateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nth = s.chars().nth(0).unwrap_or('N');
        let sign = match nth {
            'N' => 1,
            'E' => 1,
            'W' => -1,
            'S' => -1,
            _ => return Err(ParseCoordinateError),
        };
        let offset = match nth {
            'E' => 1,
            'W' => 1,
            _ => 0,
        };
        let deg = &s[1..3 + offset];
        let deg = sign * deg.parse::<i32>().map_err(|_| ParseCoordinateError)?;
        let min = &s[3 + offset..5 + offset];
        let min = min.parse::<u32>().map_err(|_| ParseCoordinateError)?;
        let sec = &s[5 + offset..];
        let sec = sec.parse::<f32>().map_err(|_| ParseCoordinateError)?;
        Ok(Coordinate {
            degrees: deg,
            minutes: min,
            seconds: sec,
        })
    }
}

impl Coordinate {
    fn to_decimal_degrees(&self) -> f32 {
        let minutes: f32 = self.minutes as f32 / 60.;
        let seconds = self.seconds / 3600.;
        let degrees = self.degrees as f32;
        degrees + minutes + seconds
    }
}

#[derive(Debug, PartialEq)]
struct Position {
    lat: Coordinate,
    lon: Coordinate,
}

impl FromStr for Position {
    type Err = ParsePositionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (lat, long): (&str, &str) = s.split_once(" ").unwrap();
        let lat = Coordinate::from_str(lat).unwrap();
        let lon = Coordinate::from_str(long).unwrap();
        Ok(Position { lat, lon })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Waypoint {
    #[serde(rename = "Type")]
    waypoint_type: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Ident")]
    ident: String,
    #[serde(rename = "Latitude")]
    latitude: f32,
    #[serde(rename = "Longitude")]
    longitude: f32,
    #[serde(rename = "Elevation")]
    elevation: Option<f32>,
    #[serde(rename = "Magnetic Declination")]
    magnetic_declination: Option<f32>,
    #[serde(rename = "Tags")]
    tags: Option<String>,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "Region")]
    region: Option<String>,
    #[serde(rename = "Visible From")]
    visible_from: Option<i32>,
    #[serde(rename = "Last Edit")]
    last_edit: Option<String>,
    #[serde(rename = "Import Filename")]
    import_filename: Option<String>,
}

impl Waypoint {
    fn from_position(p: &Position, name: &str, elevation: Option<f32>) -> Result<Self, Box<dyn Error>> {
        let lat = p.lat.to_decimal_degrees();
        let lon = p.lon.to_decimal_degrees();
        Ok(Waypoint {
            waypoint_type: "Airstrip".to_owned(),
            name: name.to_owned(),
            ident: name.to_owned(),
            latitude: lat,
            longitude: lon,
            elevation,
            magnetic_declination: None,
            tags: None,
            description: None,
            region: Some("EP".to_owned()),
            visible_from: None,
            last_edit: None,
            import_filename: Some("skydemon_PL_missing.airfields.xml".to_owned()),
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let binding = fs::read_to_string("skydemon_PL_missing.airfields.xml").unwrap();
    let data = binding.as_str();
    let doc = Document::parse(data)?;
    let mut writer = Writer::from_path("userpoints.csv")?;
    // let mut writer = Writer::from_writer(vec![]);
    let airports = doc
        .descendants()
        .filter(|e| e.tag_name() == "Airfield".into());
    for airport in airports {
        let name = airport.attribute("Name").unwrap();
        let position = airport.attribute("Position").unwrap();
        let elevation = match airport.attribute("Elevation") {
            Some(s) => {
                let f: Option<f32> = match s.parse() {
                    Ok(t) => Some(t),
                    _ => None
                };
                f
            }
            _ => None
        };
        let position = Position::from_str(position).unwrap();
        let waypoint = Waypoint::from_position(&position, name, elevation)?;
        println!("{:?}", waypoint);
        writer.serialize(waypoint)?;
    }

    Ok(())
}
