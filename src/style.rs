use crate::embed_asset;
use crate::prelude::*;
use bevy::prelude::*;

const STYLE_DB_TABLE: &str = "Style";
const DEFAULT_FONT_PATH: &str = "embedded://assets/fonts/Ithaca/Ithaca-LVB75.ttf";
const DEFAULT_TEXT_COLOR: Color = Color::srgb_u8(0xe0, 0xde, 0xf4);

pub struct StylePlugin;

impl Plugin for StylePlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/fonts/Ithaca/Ithaca-LVB75.ttf");

        app.add_systems(Startup, add_style);
    }
}

pub fn add_style(mut commands: Commands, database: Res<Database>, asset_server: Res<AssetServer>) {
    commands.insert_resource(Style::from_database(
        database.into_inner(),
        asset_server.into_inner(),
    ));
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Style {
    pub font: Handle<Font>,
    pub text_color: Color,
    //background_color: Color,
}

impl Style {
    pub fn font(&self, font_size: f32) -> TextFont {
        TextFont {
            font: self.font.clone(),
            font_size,
            ..default()
        }
    }

    pub fn from_database(db: &Database, asset_server: &AssetServer) -> Self {
        let font_path: String =
            db.get_kv_table_direct_or_default(STYLE_DB_TABLE, "font", DEFAULT_FONT_PATH);

        Self {
            font: asset_server.load(font_path),
            text_color: db.get_kv_table_or_default(
                STYLE_DB_TABLE,
                "text_color",
                DEFAULT_TEXT_COLOR,
            ),
        }
    }

    pub fn to_database(
        &self,
        db: &Database,
        asset_server: &AssetServer,
    ) -> Result<(), crate::database::SetKvError> {
        let asset_path = asset_server
            .get_path(self.font.id())
            .expect("The font should have a file path!");

        db.set_kv_table(STYLE_DB_TABLE, "font", asset_path.path())?;
        db.set_kv_table(STYLE_DB_TABLE, "text_color", self.text_color)?;

        Ok(())
    }
}
