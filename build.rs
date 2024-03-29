use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("drawing_descriptor.bin"))
        .compile(&["proto/drawing.proto"], &["proto"])
        .unwrap();

    tonic_build::compile_protos("proto/drawing.proto").unwrap();
}
