use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

fn main() {
    // tries to get an instance to the systems Vulkan installation
    let instance =
        Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");
    // returns the first device no matter what (if one exists). First is not necessarily the best.
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");

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

    println!("Hello, world succeeded!");
}
