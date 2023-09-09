use bevy_ecs::component::ComponentId;

#[derive(Clone, Copy, Debug)]
pub enum AndFilter {
    With(ComponentId),
    Without(ComponentId),
    Changed(ComponentId),
    Added(ComponentId),
}
#[derive(Clone, Debug)]
pub struct AndFilters(pub Vec<AndFilter>);
#[derive(Clone, Debug)]
pub struct OrFilters(pub Vec<AndFilters>);
