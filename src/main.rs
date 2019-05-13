use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::sync::GpuFuture;

//winit window creation
use winit::{EventsLoop, WindowBuilder};

/// Minimal Vulkano implementation based on the docs here: http://vulkano.rs/guide/example-operation
fn main() {
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

    // submit and sync. Returns an async "GpuFuture" object

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

    println!("Hello, world succeeded!");
}
