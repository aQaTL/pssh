fn main() {
	println!("cargo:rustc-link-lib=dylib=../../plugin-sdk/target/release/plugin_sdk.dll");
}
