use crate::Resources;

pub trait System {
    fn update(&self, resources: &Resources);
}
