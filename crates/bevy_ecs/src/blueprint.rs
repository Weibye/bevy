use crate::system::EntityCommands;

/// Trait that allows buidling collection of entities in a directed manner.
pub trait EntityBlueprint {
    fn build(self, entity: &mut EntityCommands);
}
