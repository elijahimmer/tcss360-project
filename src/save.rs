use crate::prelude::*;
use bevy::prelude::*;

use crate::sky::SkySettings;
use bevy_ecs_tilemap::prelude::TileBundle;
use std::fs::File;
use std::io::Write;

pub struct SavePlugin;
impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveTimer>()
            .register_type::<Save>()
            .add_systems(Update, save_game);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Save;

#[derive(Resource)]
struct SaveTimer(Timer);

impl Default for SaveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

fn load_game(world: &mut World) {}

fn save_game(world: &mut World) {
    let should_save = {
        let delta = world.get_resource_mut::<Time>().unwrap().delta();
        world
            .get_resource_mut::<SaveTimer>()
            .unwrap()
            .0
            .tick(delta)
            .just_finished()
    };

    if should_save {
        // TODO: Implement game saving
    }
}
