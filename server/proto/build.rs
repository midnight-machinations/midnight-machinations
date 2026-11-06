use std::{fs, io, path::Path};

/// Generates Rust types and Twirp server stubs from the protos in /proto.
/// The TypeScript client is generated separately with `mask proto`.
fn main() -> io::Result<()> {
    let proto_root = Path::new("../../proto");
    let mut protos = Vec::new();
    collect_protos(&proto_root.join("mafia"), &mut protos)?;

    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc_bin_vendored::protoc_bin_path().map_err(io::Error::other)?);
    twurst_build::TwirpBuilder::from_prost(config)
        .with_server()
        .compile_protos(&protos, &[proto_root])
}

fn collect_protos(dir: &Path, protos: &mut Vec<std::path::PathBuf>) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", dir.display());
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_protos(&path, protos)?;
        } else if path.extension().is_some_and(|extension| extension == "proto") {
            protos.push(path);
        }
    }
    Ok(())
}
