use bevy::prelude::*;

#[derive(Debug, Resource, Clone, Copy)]
pub struct CurrentClientId(pub u64);
