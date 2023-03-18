fn main() {
	println!("cargo:rustc-link-lib=dylib=../../target/release/pssh_sdk.dll");
}
