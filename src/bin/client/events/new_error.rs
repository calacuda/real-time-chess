use bevy::prelude::*;

#[derive(Debug, Clone, Event)]
pub struct NewError(pub String);
