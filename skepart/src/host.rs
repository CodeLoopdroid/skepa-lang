use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{RtError, RtErrorKind, RtResult, RtString};

pub trait RtHost {
    fn io_print(&mut self, text: &str) -> RtResult<()>;

    fn io_println(&mut self, text: &str) -> RtResult<()> {
        self.io_print(text)?;
        self.io_print("\n")
    }

    fn io_read_line(&mut self) -> RtResult<RtString> {
        Ok(RtString::from(""))
    }

    fn datetime_now_unix(&mut self) -> RtResult<i64> {
        Err(RtError::unsupported_builtin("datetime.nowUnix"))
    }

    fn datetime_now_millis(&mut self) -> RtResult<i64> {
        Err(RtError::unsupported_builtin("datetime.nowMillis"))
    }

    fn datetime_from_unix(&mut self, _value: i64) -> RtResult<i64> {
        Err(RtError::unsupported_builtin("datetime.fromUnix"))
    }

    fn datetime_from_millis(&mut self, _value: i64) -> RtResult<i64> {
        Err(RtError::unsupported_builtin("datetime.fromMillis"))
    }

    fn datetime_parse_unix(&mut self, _value: &str) -> RtResult<i64> {
        Err(RtError::unsupported_builtin("datetime.parseUnix"))
    }

    fn datetime_component(&mut self, _name: &str, _value: i64) -> RtResult<i64> {
        Err(RtError::unsupported_builtin("datetime.component"))
    }

    fn random_seed(&mut self, _seed: i64) -> RtResult<()> {
        Err(RtError::unsupported_builtin("random.seed"))
    }

    fn random_int(&mut self, _min: i64, _max: i64) -> RtResult<i64> {
        Err(RtError::unsupported_builtin("random.int"))
    }

    fn random_float(&mut self) -> RtResult<f64> {
        Err(RtError::unsupported_builtin("random.float"))
    }

    fn fs_exists(&mut self, _path: &str) -> RtResult<bool> {
        Err(RtError::unsupported_builtin("fs.exists"))
    }

    fn fs_read_text(&mut self, _path: &str) -> RtResult<RtString> {
        Err(RtError::unsupported_builtin("fs.readText"))
    }

    fn fs_write_text(&mut self, _path: &str, _text: &str) -> RtResult<()> {
        Err(RtError::unsupported_builtin("fs.writeText"))
    }

    fn fs_append_text(&mut self, _path: &str, _text: &str) -> RtResult<()> {
        Err(RtError::unsupported_builtin("fs.appendText"))
    }

    fn fs_mkdir_all(&mut self, _path: &str) -> RtResult<()> {
        Err(RtError::unsupported_builtin("fs.mkdirAll"))
    }

    fn fs_remove_file(&mut self, _path: &str) -> RtResult<()> {
        Err(RtError::unsupported_builtin("fs.removeFile"))
    }

    fn fs_remove_dir_all(&mut self, _path: &str) -> RtResult<()> {
        Err(RtError::unsupported_builtin("fs.removeDirAll"))
    }

    fn fs_join(&mut self, _left: &str, _right: &str) -> RtResult<RtString> {
        Err(RtError::unsupported_builtin("fs.join"))
    }

    fn os_cwd(&mut self) -> RtResult<RtString> {
        Err(RtError::unsupported_builtin("os.cwd"))
    }

    fn os_platform(&mut self) -> RtResult<RtString> {
        Err(RtError::unsupported_builtin("os.platform"))
    }

    fn os_sleep(&mut self, _millis: i64) -> RtResult<()> {
        Err(RtError::unsupported_builtin("os.sleep"))
    }

    fn os_exec_shell(&mut self, _command: &str) -> RtResult<i64> {
        Err(RtError::unsupported_builtin("os.execShell"))
    }

    fn os_exec_shell_out(&mut self, _command: &str) -> RtResult<RtString> {
        Err(RtError::unsupported_builtin("os.execShellOut"))
    }
}

pub struct NoopHost {
    random_state: u64,
}

impl Default for NoopHost {
    fn default() -> Self {
        Self {
            random_state: 0x1234_5678_9ABC_DEF0,
        }
    }
}

impl RtHost for NoopHost {
    fn io_print(&mut self, text: &str) -> RtResult<()> {
        print!("{text}");
        std::io::stdout()
            .flush()
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))?;
        Ok(())
    }

    fn datetime_now_unix(&mut self) -> RtResult<i64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))?;
        Ok(now.as_secs() as i64)
    }

    fn datetime_now_millis(&mut self) -> RtResult<i64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))?;
        Ok(now.as_millis() as i64)
    }

    fn random_seed(&mut self, seed: i64) -> RtResult<()> {
        self.random_state = seed as u64;
        Ok(())
    }

    fn random_int(&mut self, min: i64, max: i64) -> RtResult<i64> {
        if min > max {
            return Err(RtError::new(
                RtErrorKind::InvalidArgument,
                "random.int min must be <= max",
            ));
        }
        self.random_state = self
            .random_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        let span = (max - min + 1) as u64;
        Ok(min + (self.random_state % span) as i64)
    }

    fn random_float(&mut self) -> RtResult<f64> {
        self.random_state = self
            .random_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        Ok((self.random_state as f64) / (u64::MAX as f64))
    }

    fn fs_exists(&mut self, path: &str) -> RtResult<bool> {
        Ok(PathBuf::from(path).exists())
    }

    fn fs_read_text(&mut self, path: &str) -> RtResult<RtString> {
        let text = fs::read_to_string(path)
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))?;
        Ok(RtString::from(text))
    }

    fn fs_write_text(&mut self, path: &str, text: &str) -> RtResult<()> {
        fs::write(path, text)
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))
    }

    fn fs_append_text(&mut self, path: &str, text: &str) -> RtResult<()> {
        use std::io::Write as _;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))?;
        file.write_all(text.as_bytes())
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))
    }

    fn fs_mkdir_all(&mut self, path: &str) -> RtResult<()> {
        fs::create_dir_all(path)
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))
    }

    fn fs_remove_file(&mut self, path: &str) -> RtResult<()> {
        fs::remove_file(path)
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))
    }

    fn fs_remove_dir_all(&mut self, path: &str) -> RtResult<()> {
        fs::remove_dir_all(path)
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))
    }

    fn fs_join(&mut self, left: &str, right: &str) -> RtResult<RtString> {
        Ok(RtString::from(
            PathBuf::from(left)
                .join(right)
                .to_string_lossy()
                .into_owned(),
        ))
    }

    fn os_cwd(&mut self) -> RtResult<RtString> {
        std::env::current_dir()
            .map(|path| RtString::from(path.to_string_lossy().into_owned()))
            .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))
    }

    fn os_platform(&mut self) -> RtResult<RtString> {
        Ok(RtString::from(std::env::consts::OS))
    }

    fn os_sleep(&mut self, millis: i64) -> RtResult<()> {
        if millis < 0 {
            return Err(RtError::new(
                RtErrorKind::InvalidArgument,
                "os.sleep millis must be non-negative",
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(millis as u64));
        Ok(())
    }

    fn os_exec_shell(&mut self, command: &str) -> RtResult<i64> {
        let output = if cfg!(windows) {
            Command::new("cmd").args(["/C", command]).output()
        } else {
            Command::new("sh").args(["-c", command]).output()
        }
        .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))?;
        Ok(output.status.code().unwrap_or(-1) as i64)
    }

    fn os_exec_shell_out(&mut self, command: &str) -> RtResult<RtString> {
        let output = if cfg!(windows) {
            Command::new("cmd").args(["/C", command]).output()
        } else {
            Command::new("sh").args(["-c", command]).output()
        }
        .map_err(|err| RtError::new(RtErrorKind::InvalidArgument, err.to_string()))?;
        Ok(RtString::from(
            String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_string(),
        ))
    }
}
