use std::process::Output;

use anyhow::bail;

pub fn process_output(
    output: Output,
    process: &String,
    filename: &String,
    action: &str,
) -> anyhow::Result<()> {
    if output.status.success() {
        Ok(())
    } else {
        bail!("{process} failed to {action} {filename}")
    }
}
