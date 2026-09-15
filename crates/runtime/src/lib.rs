use anyhow::{Result, anyhow};

pub struct GpuRuntime {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl GpuRuntime {
    pub fn new() -> Result<Self> {
        let instance = wgpu::Instance::default();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .map_err(|e| anyhow!("failed to find GPU adapter: {e}"))?;

        let info = adapter.get_info();

        println!("GPU: {}", info.name);
        println!("Backend: {:?}", info.backend);

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("compute-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
            }))?;

        Ok(Self { device, queue })
    }

    pub fn vector_add(&self, a: &[f32], b: &[f32]) -> Result<Vec<f32>> {
        if a.len() != b.len() {
            return Err(anyhow!(
                "vectors must have equal length: {} != {}",
                a.len(),
                b.len()
            ));
        }

        if a.is_empty() {
            return Ok(Vec::new());
        }

        let buffer_size = (a.len() * std::mem::size_of::<f32>()) as u64;

        // ------------------------------------------------------------
        // 1. Create GPU buffers
        // ------------------------------------------------------------

        let a_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vector-a"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let b_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vector-b"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let result_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vector-result"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vector-result-staging"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ------------------------------------------------------------
        // 2. Upload input data to GPU
        // ------------------------------------------------------------

        self.queue
            .write_buffer(&a_buffer, 0, bytemuck::cast_slice(a));

        self.queue
            .write_buffer(&b_buffer, 0, bytemuck::cast_slice(b));

        // ------------------------------------------------------------
        // 3. Load shader
        // ------------------------------------------------------------

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("vector-add-shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/vector_add.wgsl").into()),
            });

        // ------------------------------------------------------------
        // 4. Describe bindings
        // ------------------------------------------------------------

        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("vector-add-bind-group-layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        // ------------------------------------------------------------
        // 5. Create pipeline
        // ------------------------------------------------------------

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("vector-add-pipeline-layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("vector-add-pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        // ------------------------------------------------------------
        // 6. Create bind group
        // ------------------------------------------------------------

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vector-add-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: a_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: b_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: result_buffer.as_entire_binding(),
                },
            ],
        });

        // ------------------------------------------------------------
        // 7. Dispatch GPU compute
        // ------------------------------------------------------------

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("vector-add-command-encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("vector-add-compute-pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroups = (a.len() as u32).div_ceil(256);

            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // ------------------------------------------------------------
        // 8. Copy GPU result into readable staging buffer
        // ------------------------------------------------------------

        encoder.copy_buffer_to_buffer(&result_buffer, 0, &staging_buffer, 0, buffer_size);

        self.queue.submit(Some(encoder.finish()));

        // ------------------------------------------------------------
        // 9. Wait for GPU and map result
        // ------------------------------------------------------------

        let slice = staging_buffer.slice(..);

        let (sender, receiver) = std::sync::mpsc::sync_channel(1);

        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })?;

        receiver
            .recv()?
            .map_err(|e| anyhow!("GPU buffer mapping failed: {e}"))?;

        // ------------------------------------------------------------
        // 10. Read result back into CPU memory
        // ------------------------------------------------------------

        let data = slice
            .get_mapped_range()
            .map_err(|e| anyhow!("GPU buffer read failed: {e}"))?;

        let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

        drop(data);

        staging_buffer.unmap();

        Ok(result)
    }
}
