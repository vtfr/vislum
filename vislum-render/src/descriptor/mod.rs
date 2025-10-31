use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use vulkano::descriptor_set::{DescriptorSet, allocator::DescriptorSetAllocator};

use crate::resource::ErasedResourceId;

pub struct DescriptorSetCache {
    allocator: Arc<dyn DescriptorSetAllocator>,
    sets: HashMap<Vec<ErasedResourceId>, Arc<DescriptorSet>>,
}

impl DescriptorSetCache {
    pub fn new(allocator: Arc<dyn DescriptorSetAllocator>) -> Self {
        Self {
            allocator,
            sets: HashMap::new(),
        }
    }

    /// Gets a descriptor set from the cache, or allocates a new one if it is not found.
    pub fn get_or_insert<'a>(
        &'a mut self,
        key: impl Into<Cow<'a, [ErasedResourceId]>>,
    ) -> Arc<DescriptorSet> {
        let key = key.into();

        // Try to get the set from the cache.
        if let Some(set) = self.sets.get(key.as_ref()) {
            return set.clone();
        }

        // Allocate a new set.
        // let set = self.allocator.allocate(key);
        // self.sets.insert(key, set);
        // set
        todo!()
    }

    /// Process all the dropped resources.
    ///
    /// If any descriptor set was created for this dropped resource, it will be removed from the cache.
    /// 
    /// This process is O(n * m), where n is the number of descriptor sets stored in the cache 
    /// and m is the maximum number of resources used by the descriptor sets.
    pub fn cleanup_removed_resources(
        &mut self,
        resources: impl IntoIterator<Item = ErasedResourceId>,
    ) {
        // Convert the iterator into a set for O(1) containment checks.
        let resources = resources.into_iter().collect::<HashSet<_>>();

        // Retain only the descriptor sets that do not contain any of the dropped resources.
        let retain =
            |key: &[ErasedResourceId]| !key.iter().any(|id| resources.contains(id));

        self.sets.retain(|key, _| retain(key));
    }
}
