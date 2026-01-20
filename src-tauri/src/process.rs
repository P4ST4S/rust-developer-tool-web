use std::process::Stdio;
use tokio::process::{Child, Command};

#[cfg(unix)]
pub fn create_process_group_command(program: &str, args: &[&str], dir: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    // Create a new process group
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
    
    cmd
}

#[cfg(not(unix))]
pub fn create_process_group_command(program: &str, args: &[&str], dir: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

#[cfg(unix)]
pub async fn kill_process_group(child: &mut Child) -> std::io::Result<()> {
    if let Some(pid) = child.id() {
        // Send SIGTERM to the entire process group
        unsafe {
            libc::kill(-(pid as i32), libc::SIGTERM);
        }
        
        // Give it a moment to terminate gracefully
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Check if still alive, if so send SIGKILL
        if child.try_wait()?.is_none() {
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
    }
    Ok(())
}

#[cfg(not(unix))]
pub async fn kill_process_group(child: &mut Child) -> std::io::Result<()> {
    // On Windows, try to kill the process
    child.kill().await
}
