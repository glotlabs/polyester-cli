use std::fmt;
use std::fmt::Formatter;
use std::io;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::string;

#[derive(Debug)]
pub enum Error {
    FailedToExecute(io::Error),
    FailedToReadStdout(string::FromUtf8Error),
    FailedToReadStderr(string::FromUtf8Error),
    ExitFailure {
        stdout: String,
        stderr: String,
        exit_status: Option<i32>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::FailedToExecute(err) => write!(f, "Failed to execute command: {}", err),
            Error::FailedToReadStdout(err) => write!(f, "Failed to read stdout: {}", err),
            Error::FailedToReadStderr(err) => write!(f, "Failed to read stderr: {}", err),
            Error::ExitFailure {
                stdout,
                stderr,
                exit_status,
            } => {
                let mut output = String::new();

                if let Some(exit_status) = exit_status {
                    output.push_str(&format!("Command failed with status: {}\n", exit_status));
                } else {
                    output.push_str(&format!("Command failed\n"));
                }

                if !stdout.is_empty() {
                    output.push_str(&format!("\n[stdout]\n{}\n", stdout));
                }

                if !stderr.is_empty() {
                    output.push_str(&format!("\n[stderr]\n{}\n", stderr));
                }

                write!(f, "{}", output)
            }
        }
    }
}

pub struct Config {
    pub work_dir: PathBuf,
    pub cmd: String,
    pub args: Vec<String>,
}

pub fn to_args(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

pub fn run(config: &Config) -> Result<String, Error> {
    Command::new(&config.cmd)
        .current_dir(&config.work_dir)
        .args(&config.args)
        .output()
        .map(|output| Output(output))
        .map_err(Error::FailedToExecute)
        .and_then(|output| output.read_stdout())
}

#[derive(Debug)]
pub struct Output(process::Output);

impl Output {
    pub fn read_stdout(self) -> Result<String, Error> {
        if self.0.status.success() {
            String::from_utf8(self.0.stdout).map_err(Error::FailedToReadStdout)
        } else {
            let stdout = String::from_utf8(self.0.stdout).map_err(Error::FailedToReadStdout)?;
            let stderr = String::from_utf8(self.0.stderr).map_err(Error::FailedToReadStderr)?;
            let exit_status = self.0.status.code();

            Err(Error::ExitFailure {
                stdout,
                stderr,
                exit_status,
            })
        }
    }
}
