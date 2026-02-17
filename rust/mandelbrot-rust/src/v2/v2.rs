// rendering the mandelbrot set using GPU compute shaders in Rust with wgpu

use wgpu::util::DeviceExt;
use std::time::Instant;
use std::path::Path;

struct Params {
    width: u32,
    height: u32,
    iterations: u32,
    threshold: f64,
    // range for mandelbrot set
    real_range: (f64, f64),
    imaginary_range: (f64, f64),
    output: String,
    // two colors to gradient between
    color_1: [u8; 3],
    color_2: [u8; 3],
    color_fill: [u8; 3],
    color_blend: f64, // exponent factor of color easing
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderParams {
    width: u32,
    height: u32,
    iterations: u32,
    threshold: f32,
    real_min: f32,
    real_max: f32,
    imaginary_min: f32,
    imaginary_max: f32,
    // packed RGB as 0xRRGGBB
    color_1: u32,
    color_2: u32,
    color_fill: u32,
    color_blend: f32,
    real_step: f32,
    imaginary_step: f32,
}

fn main() {
    let start_time = Instant::now();

    let p = Params {
        width: 2000,
        height: 2000,
        iterations: 200,
        threshold: 2.0,
        real_range: (-1.5, 0.5),
        imaginary_range: (-1.0, 1.0),
        output: String::from("output_rust.png"),
        color_1: [0, 0, 0],
        color_2: [255, 0, 0],
        color_fill: [0, 0, 0],
        color_blend: 0.4,
    };

    println!("=========================================================");
    println!("Information:");
    println!("Output Destination: {}", p.output);
    println!("Image Dimensions: {}px by {}px", p.width, p.height);
    println!("Iterations: {}", p.iterations);
    println!("Threshold: {}", p.threshold);
    println!("Real Range: [{}, {}]", p.real_range.0, p.real_range.1);
    println!("Imaginary Range: [{}, {}]", p.imaginary_range.0, p.imaginary_range.1);
    println!("=========================================================");

    // calculate step sizes for mapping pixels to complex coordinates
    let real_step = (p.real_range.1 - p.real_range.0).abs() / p.width as f64;
    let imaginary_step = (p.imaginary_range.1 - p.imaginary_range.0).abs() / p.height as f64;

    // pack parameters into a struct to pass to the shader
    let shader_params = ShaderParams {
        width: p.width,
        height: p.height,
        iterations: p.iterations,
        threshold: p.threshold as f32,
        real_min: p.real_range.0 as f32,
        real_max: p.real_range.1 as f32,
        imaginary_min: p.imaginary_range.0 as f32,
        imaginary_max: p.imaginary_range.1 as f32,
        color_1: ((p.color_1[0] as u32) << 16) | ((p.color_1[1] as u32) << 8) | (p.color_1[2] as u32),
        color_2: ((p.color_2[0] as u32) << 16) | ((p.color_2[1] as u32) << 8) | (p.color_2[2] as u32),
        color_fill: ((p.color_fill[0] as u32) << 16) | ((p.color_fill[1] as u32) << 8) | (p.color_fill[2] as u32),
        color_blend: p.color_blend as f32,
        real_step: real_step as f32,
        imaginary_step: imaginary_step as f32,
    };

    // run the GPU computation asynchronously and block until it's done
    pollster::block_on(run(shader_params, &p.output));

    println!("Total time taken: {:.2?}", start_time.elapsed());
}


async fn run(shader_params: ShaderParams, filename: &str) {
    let setup_start_time = Instant::now();
    // ---- setup GPU connection ----
    let instance = wgpu::Instance::default();
    // adapter: physical GPU
    let adapter = instance.request_adapter(&Default::default()).await.unwrap();
    // device: logical connection to GPU
    // queue: command queue for sending jobs to GPU
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor::default(),
            None,
        )
        .await
        .unwrap();

    println!("Running on: {:?}", adapter.get_info().name);
    // ---- create buffers ----

    // output buffer size: we need one u32 per pixel to store the color (in RGBA8 format)
    let output_buffer_size = (shader_params.width * shader_params.height * std::mem::size_of::<u32>() as u32) as wgpu::BufferAddress;

    // output buffer (GPU): where the shader will write the pixel colors
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::STORAGE   // allows the shader to write to this buffer
             | wgpu::BufferUsages::COPY_SRC, // allows GPU to copy data out of this buffer
        mapped_at_creation: false,
    });

    // staging buffer (CPU): we will copy the data from the output buffer to this buffer so we can read it on the CPU
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::MAP_READ  // allows us to read this buffer on the CPU
             | wgpu::BufferUsages::COPY_DST, // allows us to copy data into this buffer from the GPU
        mapped_at_creation: false,
    });

    // params buffer (Uniform): holds the shader parameters
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Params Buffer"),
        contents: bytemuck::bytes_of(&shader_params),
        usage: wgpu::BufferUsages::UNIFORM   // allows the shader to read this buffer as uniform data (read-only, same for all threads)
             | wgpu::BufferUsages::COPY_DST, // allows us to copy data into this buffer from the CPU
    });

    // ---- pipeline setup ----
    // shader module: compiles our WGSL shader code so we can use it in a pipeline
    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    // compute pipeline: describes the shader and its interface (which we defined in the bind group layout)
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: None, // Auto-detect layout from shader
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    // pipeline layout: describes the "interface" between the shader and Rust code (e.g. what buffers we will use)
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);

    // bind group: connects the actual buffer we created to the bind group layout (the "interface" the shader expects)
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: output_buffer.as_entire_binding(), // bind the output buffer to binding 0
        }, wgpu::BindGroupEntry {
            binding: 1,
            resource: params_buffer.as_entire_binding(), // bind the params buffer to binding 1
        }],
    });

    println!("GPU setup completed in {:.2?}", setup_start_time.elapsed());
    let compute_start_time = Instant::now();

    // ---- execution (the encoder) ----
    // command encoder: a "builder" for GPU commands; we will record commands into this and then submit it to the GPU
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);

        // dispatch the compute shader: this tells the GPU how many threads to run
        let dispatch_x = shader_params.width.div_ceil(8); // round up to nearest multiple of 8
        let dispatch_y = shader_params.height.div_ceil(8); // round up to nearest multiple of 8

        cpass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
    }

    // copy result from Storage (VRAM) -> Staging (RAM-ish)
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, output_buffer.size());

    // submit the recorded commands to the GPU for execution
    queue.submit(Some(encoder.finish()));

    // ---- read results ----
    // in wgpu, to read the results of a GPU computation on the CPU, you have to "map" the buffer (ask the GPU to make it available to the CPU), 
    // and then read the data once it's ready
    // this is an asynchronous operation because the GPU works in parallel and we don't want to block the CPU while waiting
    let buffer_slice = staging_buffer.slice(..);
    
    // create a "oneshot" channel to listen for when the GPU is done mapping
    let (sender, receiver) = futures::channel::oneshot::channel();
    
    // tell the GPU to notify us (by sending a message on the channel) once the buffer is ready to be read on the CPU
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // freeze the main thread until the GPU has finished and the buffer is ready to be read
    // in a real application, we would probably want to do other work on the CPU while waiting
    device.poll(wgpu::Maintain::Wait);

    // check if the mapping succeeded
    if let Ok(Ok(())) = receiver.await {
        println!("GPU computation completed in {:.2?}", compute_start_time.elapsed());
        let image_start_time = Instant::now();
        // get the mapped data as a slice of bytes
        let data = buffer_slice.get_mapped_range();

        // the data is a raw byte slice, but we know it's actually a sequence of u32 values (one per pixel) representing colors in RGBA8 format
        let raw_bytes = data.to_vec(); 

        // shader packs 0xAABBGGRR (little endian u32), the bytes are already RGBA
        if let Some(img) = image::RgbaImage::from_raw(shader_params.width, shader_params.height, raw_bytes) {
            img.save(Path::new(filename)).expect("Failed to save image!");
        } else {
            println!("Failed to create image from raw data!");
        }
        println!("Image creation and saving completed in {:.2?}", image_start_time.elapsed());

        // cleanup: unmap the buffer and drop the data slice to free resources
        drop(data);
        staging_buffer.unmap();
    } else {
        println!("Failed to read data from GPU!");
    }
}
