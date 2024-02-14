#![allow(unused, non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::slice;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

struct Instance {
	width: usize,
	height: usize,
	pixels: Vec<u32>,
}

#[no_mangle]
pub extern "C" fn f0r_init() {}

#[no_mangle]
pub extern "C" fn f0r_deinit() {}

#[no_mangle]
pub extern "C" fn f0r_get_plugin_info(info: *mut f0r_plugin_info_t) {
	unsafe {
		let info = &mut *info;
		info.name = b"pixel_sorter\0".as_ptr() as _;
		info.author = b"Yoplitein\0".as_ptr() as _;
		info.explanation = b"Sorts pixels and doesn't afraid of anything\0".as_ptr() as _;
		info.plugin_type = F0R_PLUGIN_TYPE_FILTER as _;
		info.color_model = F0R_COLOR_MODEL_RGBA8888 as _;
		info.frei0r_version = FREI0R_MAJOR_VERSION as _;
		info.major_version = 0;
		info.minor_version = 0;
		info.num_params = 0;
	}
}

#[no_mangle]
pub extern "C" fn f0r_get_param_info(info: *mut f0r_param_info_t, index: i32) {}

#[no_mangle]
pub extern "C" fn f0r_get_param_value(inst: f0r_instance_t, param: f0r_param_t, index: i32) {}

#[no_mangle]
pub extern "C" fn f0r_set_param_value(inst: f0r_instance_t, param: f0r_param_t, index: i32) {}

#[no_mangle]
pub extern "C" fn f0r_construct(width: u32, height: u32) -> f0r_instance_t {
	let (width, height) = (width as _, height as _);
	let instance = Instance {
		width,
		height,
		pixels: Vec::with_capacity(width * height),
	};
	let instance = Box::leak(Box::new(instance));
	instance as *mut Instance as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn f0r_destruct(inst: f0r_instance_t) {
	unsafe {
		Box::from_raw(inst as *mut Instance);
	}
}

#[no_mangle]
pub extern "C" fn f0r_update(inst: f0r_instance_t, time: f64, input: *const u32, output: *mut u32) {
	let (inst, input, output) = unsafe {
		let inst = &mut *(inst as *mut Instance);
		let len = inst.width * inst.height;
		let input = slice::from_raw_parts(input, len);
		let output = slice::from_raw_parts_mut(output, len);
		(inst, input, output)
	};

	for y in 0 .. inst.height {
		inst.pixels.clear();
		for x in 0 .. inst.width {
			inst.pixels.push(input[inst.width * y + x]);
		}
		inst.pixels.sort();
		for x in 0 .. inst.width {
			output[inst.width * y + x] = inst.pixels[x];
		}
	}
}
