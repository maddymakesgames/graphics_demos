use bytemuck::{Pod, Zeroable};
use egui::{color_picker::color_edit_button_rgb, ComboBox, Context, Slider, Ui, Window};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferUsages, Device, FragmentState, FrontFace, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexBufferLayout,
    VertexState, VertexStepMode,
};

pub struct WgpuDemos {
    curr_demo: usize,
    demos: Vec<Box<dyn WgpuDemo>>,
}

impl WgpuDemos {
    pub fn new(device: &Device, output_format: TextureFormat) -> WgpuDemos {
        WgpuDemos {
            curr_demo: 0,
            demos: vec![OrbitDemo::new(device, output_format).convert()],
        }
    }

    pub fn ui(&mut self, ctx: &Context) {
        Window::new("Demo Settings").show(ctx, |ui| {
            ui.heading("Demo");
            ComboBox::from_id_source("Demo").show_index(
                ui,
                &mut self.curr_demo,
                self.demos.len(),
                |index| self.demos[index].name().to_string(),
            );

            ui.separator();

            ui.heading("Settings");
            self.demos[self.curr_demo].ui(ui)
        });
    }

    pub fn render<'rpass>(&'rpass self, queue: &Queue, render_pass: &mut RenderPass<'rpass>) {
        self.demos[self.curr_demo].render(queue, render_pass)
    }
}

pub trait WgpuDemo {
    fn convert(self) -> Box<dyn WgpuDemo>
    where
        Self: Sized + 'static,
    {
        Box::new(self) as Box<dyn WgpuDemo>
    }
    fn name(&self) -> &'static str;
    fn ui(&mut self, ui: &mut Ui);
    fn render<'rpass>(&'rpass self, queue: &Queue, render_pass: &mut RenderPass<'rpass>);
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex2D {
    pos: [f32; 2],
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex3D {
    pos: [f32; 3],
}

pub struct OrbitDemo {
    data: OrbitDemoUniform,
    render_pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    vertex_buffer: Buffer,
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct OrbitDemoUniform {
    // align 16
    a: u32,               // offset 0, align 4, size 4
    b: u32,               // offset 4, align 4, size 4
    timestep: u32,        // offset 8, align 4, size 4
    time: f32,            // offset 12, align 4, size 4
    color: [f32; 3],      // offset 16, align 16, size 12
    show_line: u32,       // offset 28, align 4, size 4
    line_color: [f32; 3], // offset 32, align 16, size 12
    __padding: u32,       // offset 44, align 4, size 4
}

impl OrbitDemo {
    fn new(device: &Device, output_format: TextureFormat) -> OrbitDemo {
        let data = OrbitDemoUniform {
            a: 10,
            b: 10,
            timestep: 15,
            time: 0.0,
            color: [1.0, 1.0, 1.0],
            // __padding: 0,
            show_line: 0,
            line_color: [0.5, 0.5, 0.5],
            __padding: 0,
        };

        let shader = include_wgsl!("orbit.wgsl");
        let module = device.create_shader_module(&shader);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                Vertex2D { pos: [-1.0, -1.0] },
                Vertex2D { pos: [-1.0, 1.0] },
                Vertex2D { pos: [1.0, 1.0] },
                Vertex2D { pos: [1.0, 1.0] },
                Vertex2D { pos: [1.0, -1.0] },
                Vertex2D { pos: [-1.0, -1.0] },
            ]),
            usage: BufferUsages::VERTEX,
        });
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                entry_point: "vs_main",
                module: &module,
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex2D>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x2],
                }],
            },
            fragment: Some(FragmentState {
                entry_point: "fs_main",
                module: &module,
                targets: &[output_format.into()],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                unclipped_depth: false,
                conservative: false,
                cull_mode: None,
                front_face: FrontFace::Cw,
                polygon_mode: PolygonMode::Fill,
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },
            multiview: None,
        });

        Self {
            data,
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
        }
    }
}

impl WgpuDemo for OrbitDemo {
    fn name(&self) -> &'static str {
        "Orbit"
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.label("a: ");
        ui.add(Slider::new(&mut self.data.a, 1..=50));
        ui.label("b: ");
        ui.add(Slider::new(&mut self.data.b, 1..=50));

        ui.label("speed: ");
        ui.add(Slider::new(&mut self.data.timestep, 0..=100));

        ui.label("dot color: ");
        color_edit_button_rgb(ui, &mut self.data.color);

        let mut show_line = self.data.show_line != 0;
        ui.checkbox(&mut show_line, "show orbit");
        self.data.show_line = show_line as u32;
        if show_line {
            ui.label("orbit color: ");
            color_edit_button_rgb(ui, &mut self.data.line_color);
        }

        self.data.time += f32::to_radians(self.data.timestep as f32) / 10.0;
    }

    fn render<'rpass>(&'rpass self, queue: &Queue, render_pass: &mut RenderPass<'rpass>) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.data]));

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}
