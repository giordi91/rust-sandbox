use std::collections::HashMap;

use super::super::handle;
use super::api;

#[derive(Default)]
pub struct BufferManager {
    buffers_mapper: HashMap<u64, wgpu::Buffer>,
    buffer_counter: u64,
}

impl BufferManager {
    pub fn create_buffer_with_data(
        &mut self,
        data: &[u8],
        usage_flags: wgpu::BufferUsage,
        gpu_interfaces: &api::GPUInterfaces,
    ) -> handle::ResourceHandle {
        let wgpu_buffer = gpu_interfaces
            .device
            .create_buffer_with_data(data, usage_flags);

        //build the handle
        self.buffer_counter += 1;
        self.buffers_mapper.insert(self.buffer_counter, wgpu_buffer);
        handle::ResourceHandle::new(handle::ResourceHandleType::Buffer, self.buffer_counter)
    }

    pub fn get_buffer_from_handle(&self, raw_handle: u64) -> &wgpu::Buffer {
        let curr_handle = handle::ResourceHandle::from_data(raw_handle);
        let handle_type = curr_handle.get_type();
        assert!(
            handle_type == handle::ResourceHandleType::Buffer,
            "provided handle is not a Buffer handle"
        );
        self.buffers_mapper.get(&curr_handle.get_value()).unwrap()
    }
}
