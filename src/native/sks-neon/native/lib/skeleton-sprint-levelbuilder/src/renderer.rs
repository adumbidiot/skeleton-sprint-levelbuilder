use crate::WINDOW_HEIGHT;
use crate::WINDOW_WIDTH;
use futures::task::SpawnExt;
use std::convert::TryInto;
use std::io::Write;

#[derive(Debug)]
pub enum RenderError {
    MissingAdapter,
    RequestDevice(wgpu::RequestDeviceError),

    Io(std::io::Error),
}

impl From<wgpu::RequestDeviceError> for RenderError {
    fn from(e: wgpu::RequestDeviceError) -> Self {
        RenderError::RequestDevice(e)
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

    pub iced_renderer: iced_wgpu::Renderer,
    pub iced_staging_belt: wgpu::util::StagingBelt,
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

        let iced_staging_belt = wgpu::util::StagingBelt::new(5 * 1024);
        let iced_renderer_settings = iced_wgpu::Settings {
            format: output_texture_format,
            default_font: Some(crate::FONT_DATA),
            default_text_size: 20, // Default size
            antialiasing: None,
        };
        let iced_backend = iced_wgpu::Backend::new(&wgpu_device, iced_renderer_settings);
        let iced_renderer = iced_wgpu::Renderer::new(iced_backend);

        Ok(Renderer {
            wgpu_instance,
            wgpu_adapter,
            wgpu_device,
            wgpu_queue,

            wgpu_output_buffer_dimensions,
            wgpu_output_texture_extent,
            wgpu_output_buffer,
            wgpu_output_texture,

            iced_renderer,
            iced_staging_belt,
        })
    }

    // Might be useful later
    #[allow(dead_code)]
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

    pub fn draw_ui(
        &mut self,
        iced_state: &iced_native::program::State<crate::ui::UiApp>,
        iced_debug: &iced_native::Debug,
    ) -> Result<(), RenderError> {
        let mut encoder = self
            .wgpu_device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let viewport_size =
                iced_core::Size::new(crate::WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32);
            let viewport = iced_wgpu::Viewport::with_physical_size(viewport_size, 1.0);
            let _mouse_interaction = self.iced_renderer.backend_mut().draw(
                &self.wgpu_device,
                &mut self.iced_staging_belt,
                &mut encoder,
                &self.wgpu_output_texture.create_view(&Default::default()),
                &viewport,
                iced_state.primitive(),
                &iced_debug.overlay(),
            );

            self.iced_staging_belt.finish();
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

        let mut local_pool = futures::executor::LocalPool::new();
        local_pool
            .spawner()
            .spawn(self.iced_staging_belt.recall())
            .expect("Recall staging buffers");

        local_pool.run_until_stalled();

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
