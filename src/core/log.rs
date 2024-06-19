use anyhow::Result;
use flexi_logger::{FileSpec, Logger, WriteMode};

pub fn set_logger() -> Result<()> {
    let file = FileSpec::try_from("./log")?;
    Logger::try_with_str("info")
        .unwrap()
        .log_to_file(file)
        .write_mode(WriteMode::BufferAndFlush)
        .start()
        .unwrap();
    Ok(())
}
