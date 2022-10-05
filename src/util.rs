use std::process::Output;

use crate::error;
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
        bail!(error!("{process} failed to {action} {filename}"))
    }
}
