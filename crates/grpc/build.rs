use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = "./protobufs";
    let proto_files: Vec<String> = fs::read_dir(dir_path)?
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_string_lossy().into_owned())
        .filter(|path| path.ends_with(".proto"))
        .collect();

    tonic_build::configure()
        .build_server(true)
        .compile_protos(&proto_files, &[dir_path])?;

    Ok(())
}
