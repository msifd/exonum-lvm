use exonum_build::{get_exonum_protobuf_files_path, protobuf_generate};

fn main() {
    println!("cargo:rerun-if-changed=src/proto/lvm.proto");
    let exonum_protos = get_exonum_protobuf_files_path();
    protobuf_generate(
        "src/proto",
        &["src/proto", &exonum_protos],
        "protobuf_mod.rs",
    );
}
