use super::error::GpuError;
use wgpu::*;

#[derive(Debug)]
pub struct GpuBuffer {
    pub buffer: Buffer,
    pub size_bytes: u64,
    pub usage: BufferUsages,
}

impl GpuBuffer {
    /// Allocate a real GPU buffer
    pub fn allocate(
        device: &Device,
        size_bytes: u64,
        usage: BufferUsages,
        label: Option<&str>,
    ) -> Result<Self, GpuError> {
        if size_bytes == 0 {
            return Err(GpuError::InvalidBufferSize {
                size: 0,
                reason: "Buffer size must be strictly greater than 0 bytes".to_string(),
            });
        }

        // Align buffer size to 4 bytes minimum
        let aligned_size = (size_bytes + 3) & !3;

        let buffer = device.create_buffer(&BufferDescriptor {
            label,
            size: aligned_size,
            usage,
            mapped_at_creation: false,
        });

        Ok(Self {
            buffer,
            size_bytes: aligned_size,
            usage,
        })
    }

    /// Upload host memory data to the GPU buffer via queue
    pub fn upload<T: bytemuck::Pod>(
        queue: &Queue,
        buffer: &Buffer,
        offset: u64,
        data: &[T],
    ) -> Result<(), GpuError> {
        if data.is_empty() {
            return Ok(());
        }

        let bytes = bytemuck::cast_slice(data);
        queue.write_buffer(buffer, offset, bytes);
        Ok(())
    }

    /// Read back GPU buffer data to host memory via a staging buffer
    pub fn readback<T: bytemuck::Pod + Clone>(
        device: &Device,
        queue: &Queue,
        src_buffer: &Buffer,
        size_bytes: u64,
    ) -> Result<Vec<T>, GpuError> {
        if size_bytes == 0 {
            return Ok(Vec::new());
        }

        let aligned_size = (size_bytes + 3) & !3;

        // 1. Create staging buffer with MAP_READ and COPY_DST
        let staging_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("GPU Readback Staging Buffer"),
            size: aligned_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 2. Encode and submit copy command
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("GPU Readback Copy Encoder"),
        });

        encoder.copy_buffer_to_buffer(src_buffer, 0, &staging_buffer, 0, aligned_size);
        queue.submit(Some(encoder.finish()));

        // 3. Map staging buffer for host reading
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();

        buffer_slice.map_async(MapMode::Read, move |res| {
            let _ = tx.send(res);
        });

        // 4. Poll device until mapping completes
        device.poll(Maintain::Wait);

        match rx.recv() {
            Ok(Ok(())) => {
                let mapped_view = buffer_slice.get_mapped_range();
                let result_slice: &[T] = bytemuck::cast_slice(&mapped_view);
                let result_vec = result_slice.to_vec();
                drop(mapped_view);
                staging_buffer.unmap();
                Ok(result_vec)
            }
            Ok(Err(e)) => Err(GpuError::ReadbackFailed(format!("GPU buffer mapping failed: {:?}", e))),
            Err(e) => Err(GpuError::ReadbackFailed(format!("Failed to receive mapping notification: {}", e))),
        }
    }
}
