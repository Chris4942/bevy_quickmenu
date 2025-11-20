use bevy::ecs::hierarchy::ChildSpawnerCommands;

use crate::types::MenuAssets;

pub trait Widget {
    fn build(self, parent: &mut ChildSpawnerCommands, assets: &MenuAssets);
}
