fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../common/protos/user.proto")?;
    tonic_build::compile_protos("../common/protos/email.proto")?;
    tonic_build::compile_protos("../common/protos/templating.proto")?;
    Ok(())
}
