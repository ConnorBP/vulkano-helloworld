#version 450

// this defines the dize that we devide our processing chunks into (64 values per thread)
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

// this is our descriptor definition
// descriptor is the descriptor binding 0 in the set 0
layout(set = 0, binding = 0) buffer Data {
    uint data[];

} buf;

void main() {
    // The current x index this worker has been assigned to
    uint idx = gl_GlobalInvocationID.x;
    // Multiply the value for our worker's index by 12 inside the buffer data
    buf.data[idx] *= 12;
}