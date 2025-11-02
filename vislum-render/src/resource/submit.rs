pub struct ResourceSubmitter {
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl ResourceSubmitter {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        Self { device, queue }
    }
}