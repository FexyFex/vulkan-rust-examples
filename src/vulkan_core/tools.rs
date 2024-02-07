use std::fs::File;
use std::io::Read;
use std::path::Path;
use ash::vk;


pub fn read_shader_code(shader_path: &Path) -> Vec<u32> {
    let spv_file = File::open(shader_path).expect("MEH");
    let byte_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();

    let mut cursor = std::io::Cursor::new(byte_code);
    let spv = ash::util::read_spv(&mut cursor).expect("MEH");
    return spv;
}


pub fn find_memory_type_index(
    memory_requirements: vk::MemoryRequirements,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    memory_property_flags: vk::MemoryPropertyFlags
) -> u32 {
    for i in 0..memory_properties.memory_type_count {
        let property_flags = memory_properties.memory_types[0].property_flags;
        let type_bits_satisfied = (memory_requirements.memory_type_bits & (1 << i)) != 0;
        if property_flags.contains(memory_property_flags) && type_bits_satisfied {
            return i
        }
    }

    panic!();
}