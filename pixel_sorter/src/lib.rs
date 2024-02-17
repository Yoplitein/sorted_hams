#![allow(unused, non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::{ffi::CStr, mem::align_of, slice};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

struct Instance {
	width: usize,
	height: usize,
	mode: SortMode,
	pixels: Vec<u32>,
}

enum SortMode {
	Vertical,
	Horizontal,
	WholeFrame,
}

impl SortMode {
	fn c_str(&self) -> &'static [u8] {
		match self {
			Self::Vertical => b"vertical\0".as_slice(),
			Self::Horizontal => b"horizontal\0".as_slice(),
			Self::WholeFrame => b"whole-frame\0".as_slice(),
		}
	}
}

#[no_mangle]
pub extern "C" fn f0r_init() -> i32 {
	1
}

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
		info.color_model = F0R_COLOR_MODEL_BGRA8888 as _;
		info.frei0r_version = FREI0R_MAJOR_VERSION as _;
		info.major_version = 0;
		info.minor_version = 0;
		info.num_params = 1;
	}
}

#[no_mangle]
pub extern "C" fn f0r_get_param_info(info: *mut f0r_param_info_t, index: i32) {
	unsafe {
		assert!(index == 0);
		(*info).name = b"mode\0".as_ptr() as _;
		(*info).type_ = F0R_PARAM_STRING as _;
		(*info).explanation = b"Sorting mode\0".as_ptr() as _;
	}
}

#[no_mangle]
pub extern "C" fn f0r_get_param_value(inst: f0r_instance_t, param: f0r_param_t, index: i32) {
	unsafe {
		assert!(index == 0);
		let inst = (inst as *mut Instance).as_mut().unwrap();
		*(param as *mut *const i8) = inst.mode.c_str().as_ptr() as _;
	}
}

#[no_mangle]
pub extern "C" fn f0r_set_param_value(inst: f0r_instance_t, param: f0r_param_t, index: i32) {
	unsafe {
		assert!(index == 0);
		let inst = (inst as *mut Instance).as_mut().unwrap();
		let param = CStr::from_ptr(*(param as *mut *const i8));
		for mode in [SortMode::Horizontal, SortMode::Vertical, SortMode::WholeFrame] {
			let modeStr = CStr::from_bytes_with_nul_unchecked(mode.c_str());
			if param == modeStr {
				inst.mode = mode;
				return;
			}
		}
		panic!("trying to set mode parameter to unrecognized value {param:?}");
	}
}

#[no_mangle]
pub extern "C" fn f0r_construct(width: u32, height: u32) -> f0r_instance_t {
	let (width, height) = (width as _, height as _);
	let instance = Instance {
		width,
		height,
		mode: SortMode::Horizontal,
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
		let inst = (inst as *mut Instance).as_mut().unwrap();
		let len = inst.width * inst.height;
		assert_eq!(input as usize & align_of::<u32>() - 1, 0, "input misaligned");
		assert_eq!(output as usize & align_of::<u32>() - 1, 0, "output misaligned");
		let input = slice::from_raw_parts(input, len);
		let output = slice::from_raw_parts_mut(output, len);
		(inst, input, output)
	};

	match inst.mode {
		SortMode::Vertical => {
			for x in 0 .. inst.width {
				inst.pixels.clear();
				for y in 0 .. inst.height {
					inst.pixels.push(input[inst.width * y + x]);
				}
				inst.pixels.sort();
				for y in 0 .. inst.height {
					output[inst.width * y + x] = inst.pixels[y];
				}
			}
		},
		SortMode::Horizontal => {
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
		},
		SortMode::WholeFrame => {
			inst.pixels.clear();
			inst.pixels.extend(input);
			inst.pixels.sort();
			output.copy_from_slice(&inst.pixels);
		},
	}
}
