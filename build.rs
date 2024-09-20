fn main() {
    // Instruct Cargo to pass linker arguments to rustc.
    println!("cargo:rustc-link-arg=resources.res");
}
