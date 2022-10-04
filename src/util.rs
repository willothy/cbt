use std::{io, process::Output};

pub fn process_output(output: Output, filename: &String, action: &str) -> io::Result<()> {
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to {} {}", action, filename),
        ))
    }
}
