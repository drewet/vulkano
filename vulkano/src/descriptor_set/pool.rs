use std::mem;
use std::ptr;
use std::sync::Arc;

use device::Device;

use OomError;
use VulkanObject;
use VulkanPointers;
use check_errors;
use vk;

/// Pool from which descriptor sets are allocated from.
///
/// A pool has a maximum number of descriptor sets and a maximum number of descriptors (one value
/// per descriptor type) it can allocate.
pub struct DescriptorPool {
    pool: vk::DescriptorPool,
    device: Arc<Device>,
}

impl DescriptorPool {
    /// Initializes a new pool.
    // FIXME: capacity of the pool
    pub fn new(device: &Arc<Device>) -> Result<Arc<DescriptorPool>, OomError> {
        let vk = device.pointers();

        // FIXME: arbitrary
        let pool_sizes = vec![
            vk::DescriptorPoolSize {
                ty: vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                descriptorCount: 10,
            }
        ];

        let pool = unsafe {
            let infos = vk::DescriptorPoolCreateInfo {
                sType: vk::STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,   // TODO:
                maxSets: 100,       // TODO: let user choose
                poolSizeCount: pool_sizes.len() as u32,
                pPoolSizes: pool_sizes.as_ptr(),
            };

            let mut output = mem::uninitialized();
            try!(check_errors(vk.CreateDescriptorPool(device.internal_object(), &infos,
                                                      ptr::null(), &mut output)));
            output
        };

        Ok(Arc::new(DescriptorPool {
            pool: pool,
            device: device.clone(),
        }))
    }

    /// Returns the device this pool was created from.
    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

unsafe impl VulkanObject for DescriptorPool {
    type Object = vk::DescriptorPool;

    #[inline]
    fn internal_object(&self) -> vk::DescriptorPool {
        self.pool
    }
}

impl Drop for DescriptorPool {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let vk = self.device.pointers();
            vk.DestroyDescriptorPool(self.device.internal_object(), self.pool, ptr::null());
        }
    }
}

#[cfg(test)]
mod tests {
    use descriptor_set::DescriptorPool;

    #[test]
    fn create() {
        let (device, _) = gfx_dev_and_queue!();
        let _ = DescriptorPool::new(&device).unwrap();
    }

    #[test]
    fn device() {
        let (device, _) = gfx_dev_and_queue!();
        let pool = DescriptorPool::new(&device).unwrap();
        assert_eq!(&**pool.device() as *const _, &*device as *const _);
    }
}
