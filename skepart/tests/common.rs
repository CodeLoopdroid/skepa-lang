use skepart::{RtHost, RtResult, RtString};

#[derive(Default)]
pub struct RecordingHost {
    pub output: String,
    pub unix_now: i64,
    pub millis_now: i64,
    pub random_int_value: i64,
    pub random_float_value: f64,
    pub cwd: String,
    pub platform: String,
    pub read_line: String,
    pub shell_status: i64,
    pub shell_out: String,
}

impl RecordingHost {
    pub fn seeded() -> Self {
        Self {
            unix_now: 100,
            millis_now: 1234,
            random_int_value: 5,
            random_float_value: 0.25,
            cwd: "tmp/work".into(),
            platform: "test-os".into(),
            read_line: "typed line".into(),
            shell_status: 9,
            shell_out: "shell-out".into(),
            ..Self::default()
        }
    }
}

impl RtHost for RecordingHost {
    fn io_print(&mut self, text: &str) -> RtResult<()> {
        self.output.push_str(text);
        Ok(())
    }

    fn io_read_line(&mut self) -> RtResult<RtString> {
        Ok(RtString::from(self.read_line.clone()))
    }

    fn datetime_now_unix(&mut self) -> RtResult<i64> {
        Ok(self.unix_now)
    }

    fn datetime_now_millis(&mut self) -> RtResult<i64> {
        Ok(self.millis_now)
    }

    fn datetime_from_unix(&mut self, value: i64) -> RtResult<i64> {
        Ok(value + 10)
    }

    fn datetime_from_millis(&mut self, value: i64) -> RtResult<i64> {
        Ok(value + 20)
    }

    fn datetime_parse_unix(&mut self, value: &str) -> RtResult<i64> {
        Ok(value.len() as i64)
    }

    fn datetime_component(&mut self, name: &str, value: i64) -> RtResult<i64> {
        Ok(value + name.len() as i64)
    }

    fn random_seed(&mut self, _seed: i64) -> RtResult<()> {
        Ok(())
    }

    fn random_int(&mut self, _min: i64, _max: i64) -> RtResult<i64> {
        Ok(self.random_int_value)
    }

    fn random_float(&mut self) -> RtResult<f64> {
        Ok(self.random_float_value)
    }

    fn fs_exists(&mut self, path: &str) -> RtResult<bool> {
        Ok(path == "exists.txt")
    }

    fn fs_read_text(&mut self, path: &str) -> RtResult<RtString> {
        Ok(RtString::from(format!("read:{path}")))
    }

    fn fs_write_text(&mut self, path: &str, text: &str) -> RtResult<()> {
        self.output.push_str(&format!("[write {path}={text}]"));
        Ok(())
    }

    fn fs_append_text(&mut self, path: &str, text: &str) -> RtResult<()> {
        self.output.push_str(&format!("[append {path}+={text}]"));
        Ok(())
    }

    fn fs_mkdir_all(&mut self, path: &str) -> RtResult<()> {
        self.output.push_str(&format!("[mkdir {path}]"));
        Ok(())
    }

    fn fs_remove_file(&mut self, path: &str) -> RtResult<()> {
        self.output.push_str(&format!("[rmfile {path}]"));
        Ok(())
    }

    fn fs_remove_dir_all(&mut self, path: &str) -> RtResult<()> {
        self.output.push_str(&format!("[rmdir {path}]"));
        Ok(())
    }

    fn fs_join(&mut self, left: &str, right: &str) -> RtResult<RtString> {
        Ok(RtString::from(format!("{left}/{right}")))
    }

    fn os_cwd(&mut self) -> RtResult<RtString> {
        Ok(RtString::from(self.cwd.clone()))
    }

    fn os_platform(&mut self) -> RtResult<RtString> {
        Ok(RtString::from(self.platform.clone()))
    }

    fn os_sleep(&mut self, millis: i64) -> RtResult<()> {
        self.output.push_str(&format!("[sleep {millis}]"));
        Ok(())
    }

    fn os_exec_shell(&mut self, command: &str) -> RtResult<i64> {
        self.output.push_str(&format!("[sh {command}]"));
        Ok(self.shell_status)
    }

    fn os_exec_shell_out(&mut self, command: &str) -> RtResult<RtString> {
        self.output.push_str(&format!("[shout {command}]"));
        Ok(RtString::from(self.shell_out.clone()))
    }
}
