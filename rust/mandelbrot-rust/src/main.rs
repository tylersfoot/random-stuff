use wgpu::util::DeviceExt;
use nannou::prelude::*;
use std::fmt::Debug;
use std::thread;
use std::time::{Duration, Instant};
use nannou_egui::{self, egui, Egui};

struct Model {
    egui: Egui,
    compute: Compute,
    params: Uniforms,
    // camera parameters (f64 for precision)
    center_real: f64,
    center_imaginary: f64,
    zoom: f64,
    // fps calculation
    last_frame_time: Instant,
    real_fps: f64,
}

struct Compute {
    texture_view: wgpu::TextureView,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    width: u32,
    height: u32,
    fractal_type: u32,
    _padding: u32,
    julia_k: [f32; 2],
    iterations: u32,
    threshold: f32,
    real_min: f32,
    real_max: f32,
    imaginary_min: f32,
    imaginary_max: f32,
    real_step: f32,
    imaginary_step: f32,
    color_palette: u32,
    color_fill: u32,
    palette_iterations: u32, // max iterations for palette gradient
    smooth_coloring: u32, // 0 = off, 1 = on
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum FractalType {
    Mandelbrot,
    BurningShip,
    Julia,
    Tricorn,
    Buffalo,
    Celtic,
}

impl Debug for FractalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FractalType::Mandelbrot => write!(f, "Mandelbrot"),
            FractalType::BurningShip => write!(f, "Burning Ship"),
            FractalType::Julia => write!(f, "Julia"),
            FractalType::Tricorn => write!(f, "Tricorn"),
            FractalType::Buffalo => write!(f, "Buffalo"),
            FractalType::Celtic => write!(f, "Celtic"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ColorPalette {
    Sinebow,
    Rainbow,
}

fn main() {
    nannou::app(model)
        .event(event)
        .update(update)
        .view(view)
        .run();
}

fn model(app: &App) -> Model {
    // initial window size
    let width = 1600;
    let height = 1000;

    let window_id = app.new_window()
        .size(width, height)
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    window.set_title("super awesome fractal renderer");
    let device = window.device();
    // vsync (doesn't work?)
    app.set_loop_mode(LoopMode::RefreshSync);

    let egui = Egui::from_window(&window);

    // default settings
    let zoom = 1.0;

    let params = Uniforms {
        width,
        height,
        fractal_type: 0,
        _padding: 0,
        julia_k: [0.0, 0.6],
        iterations: 200,
        threshold: 2.0,
        // these will be overwritten on the first frame
        real_min: 0.0,
        real_max: 0.0,
        imaginary_min: 0.0,
        imaginary_max: 0.0,
        real_step: 0.0,
        imaginary_step: 0.0,
        color_palette: 0,
        color_fill: 0x000000, // black
        palette_iterations: 200,
        smooth_coloring: 1, // on by default
        
    };

    // ---- create buffers ----
    let texture = wgpu::TextureBuilder::new()
        .size([width, height])
        // standard format for screens (R, G, B, A, 8 bits each)
        .format(wgpu::TextureFormat::Rgba8Unorm)
        // usage: WRITE (Storage) + READ (Texture Binding for drawing)
        .usage(wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING)
        .build(device);
    let texture_view = texture.view().build();

    // uniform buffer: holds the shader parameters
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM   // allows the shader to read this buffer as uniform data (read-only, same for all threads)
             | wgpu::BufferUsages::COPY_DST, // allows us to copy data into this buffer from the CPU
    });

    // ---- pipeline setup ----
    // shader module: compiles our WGSL shader code so we can use it in a pipeline
    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    // pipeline layout: describes the "interface" between the shader and Rust code (e.g. what buffers we will use)
    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
        .storage_texture(
            wgpu::ShaderStages::COMPUTE,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureViewDimension::D2,
            wgpu::StorageTextureAccess::WriteOnly,
        )
        .uniform_buffer(wgpu::ShaderStages::COMPUTE, false)
        .build(device);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        }, wgpu::BindGroupEntry {
            binding: 1,
            resource: uniform_buffer.as_entire_binding(), // bind the uniform buffer to binding 1
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    // compute pipeline: describes the shader and its interface (which we defined in the bind group layout)
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main"
    });

    let compute = Compute {
        texture_view,
        uniform_buffer,
        bind_group,
        pipeline: compute_pipeline,
    };

    Model {
        egui,
        compute,
        params,
        zoom,
        last_frame_time: Instant::now(),
        real_fps: 60.0,
        // these will be overwritten on the first frame
        center_real: -0.5,
        center_imaginary: 0.0,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    // fps calculation
    let now = Instant::now();
    let duration = now.duration_since(model.last_frame_time);
    let delta_secs = duration.as_secs_f64();
    model.last_frame_time = now;
    // simple moving average to smooth out FPS display
    if delta_secs > 0.0 {
        let current_fps = 1.0 / delta_secs;
        model.real_fps = (model.real_fps * 0.9) + (current_fps * 0.1);
    }

    let egui = &mut model.egui;
    let params = &mut model.params;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    let mut view_changed = false;

    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.spacing_mut().slider_width = 250.0; // make sliders wider for better precision

        ui.label("Fractal Type:");
        let mut selected_fractal_type = match params.fractal_type {
            0 => FractalType::Mandelbrot,
            1 => FractalType::BurningShip,
            2 => FractalType::Julia,
            3 => FractalType::Tricorn,
            4 => FractalType::Buffalo,
            5 => FractalType::Celtic,
            _ => FractalType::Mandelbrot, // default/fallback
        };
        let before_fractal_type = selected_fractal_type;
        egui::ComboBox::from_id_source("fractal_type_combo")
            .selected_text(format!("{selected_fractal_type:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected_fractal_type, FractalType::Mandelbrot, "Mandelbrot");
                ui.selectable_value(&mut selected_fractal_type, FractalType::BurningShip, "Burning Ship");
                ui.selectable_value(&mut selected_fractal_type, FractalType::Julia, "Julia");
                ui.selectable_value(&mut selected_fractal_type, FractalType::Tricorn, "Tricorn");
                ui.selectable_value(&mut selected_fractal_type, FractalType::Buffalo, "Buffalo");
                ui.selectable_value(&mut selected_fractal_type, FractalType::Celtic, "Celtic");
            }
        );

        if selected_fractal_type != before_fractal_type {
            match selected_fractal_type {
                FractalType::Mandelbrot => {
                    params.fractal_type = 0;
                    params.threshold = 2.0;
                },
                FractalType::BurningShip => {
                    params.fractal_type = 1;
                    // burning ship usually uses 4 instead of 2
                    params.threshold = 4.0;
                }
                FractalType::Julia => {
                    params.fractal_type = 2;
                    params.threshold = 2.0;
                }
                FractalType::Tricorn => {
                    params.fractal_type = 3;
                    params.threshold = 2.0;
                }
                FractalType::Buffalo => {
                    params.fractal_type = 4;
                    params.threshold = 2.0;
                }
                FractalType::Celtic => {
                    params.fractal_type = 5;
                    params.threshold = 2.0;
                }
            };
            view_changed = true;
        }

        if selected_fractal_type == FractalType::Julia {
            ui.label("Julia Constant:");
            let mut julia_k = [params.julia_k[0], params.julia_k[1]];
            // fine control sliders from -2 to 2
            view_changed |= ui.add(egui::Slider::new(&mut julia_k[0], -2.0..=2.0).text("Re")).changed();
            view_changed |= ui.add(egui::Slider::new(&mut julia_k[1], -2.0..=2.0).text("Im")).changed();
            if view_changed {
                params.julia_k[0] = julia_k[0];
                params.julia_k[1] = julia_k[1];
            }
        }

        ui.separator();

        ui.label("Iterations:");
        view_changed |= ui.add(egui::Slider::new(&mut params.iterations, 1..=2000)).changed();

        ui.label("Threshold:");
        view_changed |= ui.add(egui::Slider::new(&mut params.threshold, 0.5..=10.0)).changed();

        ui.separator();

        ui.label("Color Palette:");
        let mut selected_color_palette = match params.color_palette {
            0 => ColorPalette::Sinebow,
            1 => ColorPalette::Rainbow,
            _ => ColorPalette::Sinebow, // default/fallback
        };
        let before_color_palette = selected_color_palette;
        egui::ComboBox::from_id_source("color_palette_combo")
            .selected_text(format!("{selected_color_palette:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected_color_palette, ColorPalette::Sinebow, "Sinebow");
                ui.selectable_value(&mut selected_color_palette, ColorPalette::Rainbow, "Rainbow");
            }
        );

        if selected_color_palette != before_color_palette {
            match selected_color_palette {
                ColorPalette::Sinebow => params.color_palette = 0,
                ColorPalette::Rainbow => params.color_palette = 1,
            };
            view_changed = true;
        }

        ui.label("Smooth Coloring:");
        let mut smooth_coloring = params.smooth_coloring == 1;
        if ui.checkbox(&mut smooth_coloring, "").changed() {
            params.smooth_coloring = if smooth_coloring { 1 } else { 0 };
            view_changed = true;
        }

        ui.label("Palette Iterations:");
        view_changed |= ui.add(egui::Slider::new(&mut params.palette_iterations, 0..=2000)).changed();

        ui.label("Fill Color:");
        let mut fill_color = to_rgb_f32(params.color_fill);

        if egui::color_picker::color_edit_button_rgb(ui, &mut fill_color).changed() {
            params.color_fill = to_rgb_u32(fill_color);
            view_changed = true;
        }

        let clicked = ui.button("Reset Settings").clicked();

        if clicked {
            params.iterations = 200;
            params.threshold = 2.0;
            params.palette_iterations = 200;
            params.color_fill = 0x000000;
            model.center_real = -0.5;
            model.center_imaginary = 0.0;
            model.zoom = 1.0;
            view_changed = true;
        }
    });

    // precalculate zoom & scale
    // zoom is exponential: each +1 doubles the zoom level, each -1 halves it
    // zoom - 1 so that the default zoom (1.0) corresponds to a real_zoom of 1.0 (no zoom)
    let real_zoom: f64 = 2.0_f64.powf(model.zoom - 1.0);
    let base_step = 0.005; // magic number; adjust for base zoom
    let current_step = base_step / real_zoom; // step size decreases as we zoom in

    let mut zoom_speed = 0.01;
    let move_speed_px = 4.0; // base pixels/frame speed
    let mut move_delta = move_speed_px * current_step;

    // handle key inputs

    // shift/ctrl: increase/decrease zoom & movement speed
    if app.keys.down.contains(&Key::LShift) {
        zoom_speed *= 4.0;
        move_delta *= 4.0;
    }
    if app.keys.down.contains(&Key::LControl) {
        zoom_speed *= 0.25;
        move_delta *= 0.25;
    }
    // up/down arrows: zoom in/out
    if app.keys.down.contains(&Key::Up) {
        model.zoom += zoom_speed;
        view_changed = true;
    }
    if app.keys.down.contains(&Key::Down) {
        model.zoom -= zoom_speed;
        view_changed = true;
    }
    // WASD: move view/center point
    if app.keys.down.contains(&Key::W) {
        model.center_imaginary -= move_delta;
        view_changed = true;
    }
    if app.keys.down.contains(&Key::S) {
        model.center_imaginary += move_delta;
        view_changed = true;
    }
    if app.keys.down.contains(&Key::A) {
        model.center_real -= move_delta;
        view_changed = true;
    }
    if app.keys.down.contains(&Key::D) {
        model.center_real += move_delta;
        view_changed = true;
    }

    let window = app.main_window();
    let rect = window.rect();

    let win_w = rect.w() as u32;
    let win_h = rect.h() as u32;

    // check if window size has changed
    if model.params.width != win_w || model.params.height != win_h {
        model.params.width = win_w;
        model.params.height = win_h;
        view_changed = true;
        
        // recreate texture
        let new_texture = wgpu::TextureBuilder::new()
        .size([win_w, win_h]) // new window/texture size
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .usage(wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING)
        .build(window.device());
        let new_texture_view = new_texture.view().build();

        // recreate bind group
        // we need to link the new texture to the old pipeline/buffer
        let new_bind_group = window.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &model.compute.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&new_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: model.compute.uniform_buffer.as_entire_binding(),
                },
            ],
        });

        model.compute.texture_view = new_texture_view;
        model.compute.bind_group = new_bind_group;
    }

    // force render on first few frames to ensure it's not black
    if app.elapsed_frames() < 5 {
        view_changed = true;
    }

    // only calculate and render if the view changed at all (window resize, zoom, parameter change, etc)
    if view_changed {
        // calculate view bounds based on current window size
        // this ensures a consistent visual scale regardless of window dimensions
        let half_view_real = (rect.w() as f64 / 2.0) * current_step;
        let half_view_imaginary = (rect.h() as f64 / 2.0) * current_step;

        // calculate the new view bounds based on center point
        model.params.real_min = (model.center_real - half_view_real) as f32;
        model.params.real_max = (model.center_real + half_view_real) as f32;
        model.params.imaginary_min = (model.center_imaginary - half_view_imaginary) as f32;
        model.params.imaginary_max = (model.center_imaginary + half_view_imaginary) as f32;

        model.params.real_step = current_step as f32;
        model.params.imaginary_step = current_step as f32;

        // we overwrite the buffer instead of creating a new one
        window.queue().write_buffer(&model.compute.uniform_buffer, 0, bytemuck::bytes_of(&model.params));

        // dispatch compute shader
        let mut encoder = window.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cpass.set_pipeline(&model.compute.pipeline);
            cpass.set_bind_group(0, &model.compute.bind_group, &[]);

            // dispatch the compute shader: this tells the GPU how many threads to run
            let dispatch_x = model.params.width.div_ceil(8);
            let dispatch_y = model.params.height.div_ceil(8);
            cpass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }

        // submit the compute pass to the device's queue
        window.queue().submit(Some(encoder.finish()));        
    }

    // force CPU to sleep to cap fps
    let target_frame_time = Duration::from_micros(10_000); // ~100fps cap
    let elapsed = now.elapsed();
    if elapsed < target_frame_time {
        thread::sleep(target_frame_time - elapsed);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // draw texture at (0, 0); might need to scale to fit
    draw.texture(&model.compute.texture_view);

    // fps counter
    let fps_display = model.real_fps;
    let fps_text = format!("FPS: {fps_display:.0}");
    
    let win = app.window_rect();
    draw.text(&fps_text)
        .font_size(24)
        .color(WHITE)
        .xy(win.bottom_left() + vec2(50.0, 30.0));

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn event(_app: &App, _model: &mut Model, _event: Event) {
    // TODO mouse movement?
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // let egui handle things like keyboard and mouse input
    model.egui.handle_raw_event(event);
}


fn to_rgb_f32(color: u32) -> [f32; 3] {
    // convert packed RGB u32 (0xRRGGBB) to [f32; 3] with values in [0, 1]
    [
        ((color >> 16) & 0xFF) as f32 / 255.0,
        ((color >> 8) & 0xFF) as f32 / 255.0,
        (color & 0xFF) as f32 / 255.0,
    ]
}

fn to_rgb_u32(rgb: [f32; 3]) -> u32 {
    // convert [f32; 3] with values in [0, 1] to packed RGB u32 (0xRRGGBB)
    ((rgb[0] * 255.0) as u32) << 16
    | ((rgb[1] * 255.0) as u32) << 8
    | ((rgb[2] * 255.0) as u32)
}