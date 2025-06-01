use bevy::prelude::*;

#[derive(Component, Debug)]
#[relationship(relationship_target = TrailedBy)]
pub struct Trailing(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = Trailing)]
pub struct TrailedBy(Vec<Entity>);
