use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Building {
    DumontDurville,
    Magellan,
    Bougainville,
    Darwin,
}

impl std::fmt::Display for Building {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Building::DumontDurville => write!(f, "Dumont Durville"),
            Building::Magellan => write!(f, "Magellan"),
            Building::Bougainville => write!(f, "Bougainville"),
            Building::Darwin => write!(f, "Darwin"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Rc,
    Rj,
    Level1,
    Level2,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Rc => write!(f, "RC"),
            Level::Rj => write!(f, "RJ"),
            Level::Level1 => write!(f, "R1"),
            Level::Level2 => write!(f, "R2"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Location {
    Parsed {
        building: Building,
        building_area: char,
        level: Level,
        room_number: u8,
    },
    Unparsed(String),
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Parsed {
                building,
                building_area,
                level,
                room_number,
            } => write!(f, "{} - {} - {} - {}", building, building_area, level, room_number),
            Location::Unparsed(s) => write!(f, "{}", s),
        }
    }
}
