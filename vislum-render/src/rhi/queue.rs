use std::sync::{Arc, atomic::{AtomicU8, Ordering}};

use ash::vk;
use smallvec::SmallVec;

use super::{device::Device, sync::{Fence, Semaphore}, command::CommandBuffer};

/// Represents an in-flight submission to the queue
#[derive(Debug)]
struct InFlightSubmission {
    fence: Fence,
}

/// A Vulkan queue wrapper that manages command buffer submissions and synchronization
#[derive(Debug)]
pub struct Queue {
    device: Arc<Device>,
    queue: vk::Queue,
    queue_family_index: u32,
    submissions: SmallVec<[InFlightSubmission; 4]>,
    current_submission: AtomicU8,
    max_in_flight_submissions: u8,
}

/// Description for creating a queue
#[derive(Debug, Clone)]
pub struct QueueDescription {
    pub queue_family_index: u32,
    pub queue_index: u32,
    pub max_in_flight_submissions: u8,
}

/// Information about a queue submission
#[derive(Debug, Clone)]
pub struct SubmissionInfo {
    pub command_buffers: Vec<Arc<CommandBuffer>>,
    pub wait_semaphores: Vec<Arc<Semaphore>>,
    pub wait_stages: Vec<vk::PipelineStageFlags>,
    pub signal_semaphores: Vec<Arc<Semaphore>>,
}

impl Queue {
    /// Create a new queue from a device and description
    pub fn new(device: Arc<Device>, description: QueueDescription) -> Self {
        let queue = unsafe {
            device.handle().get_device_queue(description.queue_family_index, description.queue_index)
        };

        // Create a fixed-size array for submissions with pre-allocated fences
        let submissions = (0..description.max_in_flight_submissions as usize)
            .map(|_| {
                let fence = Fence::new(Arc::clone(&device), super::sync::FenceDescription {
                    signaled: true,
                });
                InFlightSubmission { fence }
            })
            .collect();

        Self {
            device,
            queue,
            queue_family_index: description.queue_family_index,
            submissions,
            current_submission: AtomicU8::new(0),
            max_in_flight_submissions: description.max_in_flight_submissions,
        }
    }

    /// Get the Vulkan queue handle
    #[inline]
    pub fn handle(&self) -> vk::Queue {
        self.queue
    }

    /// Get the queue family index
    #[inline]
    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }

    /// Get the device this queue belongs to
    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    /// Submit command buffers to the queue
    pub fn submit(&self, submission_info: SubmissionInfo) -> Result<(), vk::Result> {
        // Get the current slot and its fence
        let current_submission = self.current_submission.load(std::sync::atomic::Ordering::Relaxed);
        let current_idx = (current_submission as usize) % (self.max_in_flight_submissions as usize);
        let submission = &self.submissions[current_idx];
        
        // Wait for the fence to be available and reset it
        submission.fence.wait_and_reset(u64::MAX);
        
        let fence = &submission.fence;

        // Prepare wait semaphores and stages
        let wait_semaphores: Vec<vk::Semaphore> = submission_info
            .wait_semaphores
            .iter()
            .map(|sem| sem.handle())
            .collect();

        let signal_semaphores: Vec<vk::Semaphore> = submission_info
            .signal_semaphores
            .iter()
            .map(|sem| sem.handle())
            .collect();

        let command_buffers: Vec<vk::CommandBuffer> = submission_info
            .command_buffers
            .iter()
            .map(|cmd| cmd.vk())
            .collect();

        // Create submission info
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&submission_info.wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        // Submit to queue
        unsafe {
            self.device.handle().queue_submit(
                self.queue,
                &[submit_info],
                fence.handle(),
            )?;
        }
        
        // Move to the next submission (monotonically increasing)
        self.current_submission.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    // the maximum number of in-flight submissions
    #[inline]
    pub fn max_in_flight_submissions(&self) -> u8 {
        self.max_in_flight_submissions
    }

    /// Get the current submission index (for debugging/monitoring)
    #[inline]
    pub fn current_submission_index(&self) -> u8 {
        let current = self.current_submission.load(std::sync::atomic::Ordering::Relaxed);
        current % self.max_in_flight_submissions
    }

}

impl Drop for Queue {
    fn drop(&mut self) {
        // Wait until all submissions are complete
        for submission in self.submissions.iter() {
            submission.fence.wait(u64::MAX);
        }
    }
}
