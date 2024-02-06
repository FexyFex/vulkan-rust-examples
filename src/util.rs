use std::fs::File;
use std::io::Read;
use std::path::Path;

pub mod image_extent;


pub fn read_shader_code(shader_path: &Path) -> Vec<u8> {
    let spv_file = File::open(shader_path).expect("MEH");
    let byte_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();
    return byte_code;
}