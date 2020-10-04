use crate::WINDOW_HEIGHT;
use crate::WINDOW_WIDTH;
use std::convert::TryInto;
use std::io::Write;

#[derive(Debug)]
pub enum RenderError {
    MissingAdapter,
    RequestDevice(wgpu::RequestDeviceError),

    CacheWrite(conrod_core::text::rt::gpu_cache::CacheWriteErr),
    Io(std::io::Error),
}

impl From<wgpu::RequestDeviceError> for RenderError {
    fn from(e: wgpu::RequestDeviceError) -> Self {
        RenderError::RequestDevice(e)
    }
}

impl From<conrod_core::text::rt::gpu_cache::CacheWriteErr> for RenderError {
    fn from(e: conrod_core::text::rt::gpu_cache::CacheWriteErr) -> Self {
        RenderError::CacheWrite(e)
    }
}

impl From<std::io::Error> for RenderError {
    fn from(e: std::io::Error) -> Self {
        RenderError::Io(e)
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::MissingAdapter => "could not locate a valid adapter".fmt(f),
            RenderError::RequestDevice(e) => e.fmt(f),
            RenderError::CacheWrite(e) => e.fmt(f),
            RenderError::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for RenderError {}

pub struct Renderer {
    pub wgpu_instance: wgpu::Instance,
    pub wgpu_adapter: wgpu::Adapter,
    pub wgpu_device: wgpu::Device,
    pub wgpu_queue: wgpu::Queue,

    pub wgpu_output_buffer_dimensions: BufferDimensions,
    pub wgpu_output_texture_extent: wgpu::Extent3d,
    pub wgpu_output_buffer: wgpu::Buffer,
    pub wgpu_output_texture: wgpu::Texture,

    pub conrod_renderer: conrod_wgpu::Renderer,
}

impl Renderer {
    pub async fn new() -> Result<Self, RenderError> {
        let backends = wgpu::BackendBit::PRIMARY | wgpu::BackendBit::SECONDARY;
        let wgpu_instance = wgpu::Instance::new(backends);

        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: None,
        };

        let wgpu_adapter = wgpu_instance
            .request_adapter(&adapter_options)
            .await
            .ok_or(RenderError::MissingAdapter)?;

        let wgpu_device_descriptor = wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            shader_validation: true,
        };

        let (wgpu_device, wgpu_queue) = wgpu_adapter
            .request_device(&wgpu_device_descriptor, None)
            .await?;

        let wgpu_output_buffer_dimensions = BufferDimensions::new(
            WINDOW_WIDTH.try_into().unwrap(),
            WINDOW_HEIGHT.try_into().unwrap(),
        );

        let wgpu_output_buffer = wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (wgpu_output_buffer_dimensions.padded_bytes_per_row
                * wgpu_output_buffer_dimensions.height) as u64,
            usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let wgpu_output_texture_extent = wgpu::Extent3d {
            width: wgpu_output_buffer_dimensions.width as u32,
            height: wgpu_output_buffer_dimensions.height as u32,
            depth: 1,
        };

        let output_texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let wgpu_output_texture = wgpu_device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu_output_texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: output_texture_format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
            label: None,
        });

        const MSAA_SAMPLES: u32 = 1;
        let conrod_renderer =
            conrod_wgpu::Renderer::new(&wgpu_device, MSAA_SAMPLES, output_texture_format);

        Ok(Renderer {
            wgpu_instance,
            wgpu_adapter,
            wgpu_device,
            wgpu_queue,

            wgpu_output_buffer_dimensions,
            wgpu_output_texture_extent,
            wgpu_output_buffer,
            wgpu_output_texture,

            conrod_renderer,
        })
    }

    pub fn create_texture(&self, image: image::RgbaImage) -> wgpu::Texture {
        let (width, height) = image.dimensions();
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = self.wgpu_device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let data = &image.into_raw()[..];

        let pixel_size_bytes = 4;
        let data_layout = wgpu::TextureDataLayout {
            offset: 0,
            bytes_per_row: width * pixel_size_bytes,
            rows_per_image: height,
        };

        let texture_copy_view = wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        };

        let extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let cmd_encoder_descriptor = wgpu::CommandEncoderDescriptor { label: None };

        let encoder = self
            .wgpu_device
            .create_command_encoder(&cmd_encoder_descriptor);
        self.wgpu_queue
            .write_texture(texture_copy_view, data, data_layout, extent);

        self.wgpu_queue.submit(Some(encoder.finish()));

        texture
    }

    pub fn draw_conrod(
        &mut self,
        ui: &conrod_core::Ui,
        image_map: &conrod_core::image::Map<conrod_wgpu::Image>,
    ) -> Result<(), RenderError> {
        let primitives = match ui.draw_if_changed() {
            None => return Ok(()),
            Some(ps) => ps,
        };

        let mut encoder = self
            .wgpu_device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let scale_factor = 1.0;
        let [win_w, win_h]: [f32; 2] = [WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32];
        let viewport = [0.0, 0.0, win_w, win_h];
        if let Some(cmd) =
            self.conrod_renderer
                .fill(&image_map, viewport, scale_factor, primitives)?
        {
            cmd.load_buffer_and_encode(&self.wgpu_device, &mut encoder);
        }

        let color_attachment_descriptor = wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &self
                .wgpu_output_texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: true,
            },
        };

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            color_attachments: &[color_attachment_descriptor],
            depth_stencil_attachment: None,
        };

        let render = self.conrod_renderer.render(&self.wgpu_device, image_map);

        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
            let slot = 0;
            render_pass.set_vertex_buffer(slot, render.vertex_buffer.slice(..));
            let instance_range = 0..1;

            for cmd in render.commands {
                match cmd {
                    conrod_wgpu::RenderPassCommand::SetPipeline { pipeline } => {
                        render_pass.set_pipeline(pipeline);
                    }
                    conrod_wgpu::RenderPassCommand::SetBindGroup { bind_group } => {
                        render_pass.set_bind_group(0, bind_group, &[]);
                    }
                    conrod_wgpu::RenderPassCommand::SetScissor {
                        top_left,
                        dimensions,
                    } => {
                        let [x, y] = top_left;
                        let [w, h] = dimensions;
                        render_pass.set_scissor_rect(x, y, w, h);
                    }
                    conrod_wgpu::RenderPassCommand::Draw { vertex_range } => {
                        render_pass.draw(vertex_range, instance_range.clone());
                    }
                }
            }
        }

        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &self.wgpu_output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::BufferCopyView {
                buffer: &self.wgpu_output_buffer,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: self.wgpu_output_buffer_dimensions.padded_bytes_per_row as u32,
                    rows_per_image: 0,
                },
            },
            self.wgpu_output_texture_extent,
        );

        self.wgpu_queue.submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn get_output_rgba_image(&self) -> Result<image::RgbaImage, RenderError> {
        let buffer_slice = self.wgpu_output_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

        futures::executor::block_on(async {
            // Poll the device in a blocking manner so that our future resolves.
            // In an actual application, `device.poll(...)` should
            // be called in an event loop or on another thread.
            self.wgpu_device.poll(wgpu::Maintain::Wait);
            buffer_future.await
        })
        .unwrap();

        let buffer_dimensions = &self.wgpu_output_buffer_dimensions;

        let padded_buffer = buffer_slice.get_mapped_range();
        let mut vec_buffer = Vec::with_capacity(buffer_dimensions.unpadded_bytes_per_row);

        for chunk in padded_buffer.chunks(buffer_dimensions.padded_bytes_per_row) {
            vec_buffer.write_all(&chunk[..buffer_dimensions.unpadded_bytes_per_row])?;
        }

        drop(padded_buffer);
        self.wgpu_output_buffer.unmap();

        let img = image::RgbaImage::from_raw(
            buffer_dimensions.width as u32,
            buffer_dimensions.height as u32,
            vec_buffer,
        )
        .unwrap();

        Ok(img)
    }
}

pub struct BufferDimensions {
    pub width: usize,
    pub height: usize,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = std::mem::size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}
