fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/gen")
        .compile(&["protos/trollbox.proto"], &["protos/"])?;
    Ok(())
}
