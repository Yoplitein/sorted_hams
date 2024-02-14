use std::path::PathBuf;

fn main() {
	let bindings = bindgen::Builder::default()
		.header("/usr/include/frei0r.h")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.ignore_functions()
		.generate()
		.expect("could not generate bindings");
	let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	bindings
		.write_to_file(out.join("bindings.rs"))
		.expect("could not write bindings");
}
