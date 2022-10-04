use std::process::Output;

use anyhow::bail;

pub fn process_output(output: Output, filename: &String, action: &str) -> anyhow::Result<()> {
    if output.status.success() {
        Ok(())
    } else {
        bail!("Failed to {} {}", action, filename)
    }
}
