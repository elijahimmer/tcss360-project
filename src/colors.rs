use crate::prelude::*;
use bevy::prelude::*;

use serde::{Deserialize, Serialize};

pub struct ColorsPlugin;

impl Plugin for ColorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_colors);
    }
}

fn add_colors(mut commands: Commands, database: Res<Database>) {
    commands.insert_resource(Colors::from_database(database.into_inner()));
}

#[derive(Resource, Reflect, Serialize, Deserialize)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct Colors {
    //background_color: Color,
    //text_color: Color,
}

impl FromDatabase for Colors {
    fn from_database(_database: &Database) -> Self {
        Self {}
    }
}

// TODO: Do this in a single transaction maybe? (don't know if it matters)
impl ToDatabase for Colors {
    fn to_database(&self, _database: &Database) -> Result<(), DatabaseError> {
        Ok(())
    }
}
