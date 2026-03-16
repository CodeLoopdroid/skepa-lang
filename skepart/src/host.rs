pub trait RtHost {
    fn io_print(&mut self, text: &str);

    fn io_println(&mut self, text: &str) {
        self.io_print(text);
        self.io_print("\n");
    }
}

#[derive(Default)]
pub struct NoopHost;

impl RtHost for NoopHost {
    fn io_print(&mut self, _text: &str) {}
}
