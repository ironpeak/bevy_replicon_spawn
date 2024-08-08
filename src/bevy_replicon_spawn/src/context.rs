use bevy_app::prelude::*;

pub trait SpawnContext {
    fn register_type(app: &mut App) -> &mut App;
}
