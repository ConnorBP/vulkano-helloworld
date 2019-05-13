use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::CommandBuffer;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::ComputePipeline;
use vulkano::sync::GpuFuture;

// winit for window creation
use winit::{EventsLoop, WindowBuilder};

/// Minimal Vulkano implementation based on the docs here: http://vulkano.rs/guide/example-operation
fn main() {
    //
    // -------------------------------------
    // Setup the needed API helpers, instances, handles, etc
    //

    // tries to get an instance to the systems Vulkan installation TODO: handle vulkan gpu driver or runtime not being installed with an informative popup error message
    let instance =
        Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");
    // returns the first device no matter what (if one exists). First is not necessarily the best.
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available"); //TODO: handle the event where none of the devices are compatible with vulkan with an informative popup error message

    // enumerates through all queue families on the physical device
    for family in physical.queue_families() {
        println!(
            "Found a queue family with {:?} queue(s)",
            family.queues_count()
        );
    }

    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, mut queues) = {
        Device::new(
            physical,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed to create device")
    };

    let queue = queues.next().unwrap();

    let data = 12;

    let buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), data)
        .expect("failed to create buffer");

    let mut content = buffer.write().unwrap();

    println!("Content before: {}", *content);

    *content *= 2;

    println!("Content after: {}", *content);

    let source_content = 0..64;
    let source = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), source_content)
        .expect("failed to create buffer");

    let dest_content = (0..64).map(|_| 0);
    let dest = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), dest_content)
        .expect("failed to create buffer");

    // create a new command buffer to run on our queue family (the one passed in here must be the same one it runs on)
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .unwrap()
        .copy_buffer(source.clone(), dest.clone())
        .unwrap()
        .build()
        .unwrap();

    // -------------------------------------
    // submit and sync. Executes the command buffer and returns a GpuFuture
    //

    let finished = command_buffer.execute(queue.clone()).unwrap();

    // using the GpuFuture
    // signals the fence, makes this thread wait for the gpu result to finish, and then flushes when it finishes
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    // after we are done waiting for the result, we can read the data from each buffer to check if it succeeded
    let src_content = source.read().unwrap();
    let dest_content = dest.read().unwrap();
    assert_eq!(&*src_content, &*dest_content);

    println!("finished running simple command buffer.");

    // -------------------------------------
    // now we are going to try running some parallel computations on a large arbitrary iteration of numbers
    // http://vulkano.rs/guide/compute-intro
    //

    //store our large iteration of values into a buffer
    let data_iter = 0..65536;
    let data_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), data_iter)
        .expect("failed to create data_iter buffer");

    // Now we create a compute pipeline (with shader programs)

    // load the generated shader interface to an object for use
    let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");

    //create the compute shader pipeline for a device
    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
            .expect("failed to create compute pipeline"),
    );

    // create the descriptor set (0) for our compute_pipeline and buffer
    // a created descriptor set can be used for any pipeline takes the same data into it's shaders
    // However, creating the descriptor set requires at least one pipeline to already exist and be provided
    let set = Arc::new(
        PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
            .add_buffer(data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    // Create a command buffer to execute the compute pipeline.
    // This is our "Dispatch Operation"

    // Create the dispatch command buffer with 1024 work groups (same as is defined in the shader code)
    // Note: The last parameter to dispatch contains the push constants, which we haven't covered yet.
    // Push constants are a way to pass a small amount of data to a shader,
    // as an alternative to putting this data in a buffer in a descriptor set.
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .unwrap()
        .dispatch([1024, 1, 1], compute_pipeline.clone(), set.clone(), ())
        .unwrap()
        .build()
        .unwrap();

    //submit the command buffer
    let finished = command_buffer.execute(queue.clone()).unwrap();

    // block thread until GPU is finished and then flush
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    println!("Finished running shader computation.");

    // retrieve the data
    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        //assert that the calculations output the expected value
        assert_eq!(*val, n as u32 * 12);
    }

    println!("Everything worked as expected.");
    println!("Hello World Complete!");
}

// generate the access code for our shader using a proc_macro
// https://docs.rs/vulkano-shaders/0.11.0/vulkano_shaders/
// ty: vertex, fragment, geometry, tess_ctrl, tess_eval, compute
// src: src code as an inline string litteral (only this or path can be defined)
// path: the path to a GLSL file relative to the cargo.toml directory root
/// Compute shader interface generated at compile-time.
/// Sets up our compute shader access functions for us as well as compiling it into SPIR-V when we build
mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/mul_12.glsl"
    }
}
