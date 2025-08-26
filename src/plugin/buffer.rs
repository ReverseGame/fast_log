use crate::appender::{Command, FastLogRecord, LogAppender};
use crate::error::LogError;

/// only write append into file
pub struct BufferAppender {
    buffer: String,
    buffer1: String,
    flag: bool,
    buffer_max_size: usize,
}

impl BufferAppender {
    pub fn new(buffer_max_size: usize) -> Result<BufferAppender, LogError> {
        Ok(Self {
            buffer: String::with_capacity(buffer_max_size + 1024 * 10),
            buffer1: "".to_string(),
            flag: false,
            buffer_max_size,
        })
    }

    pub fn get_buffer(&mut self) -> &mut String {
        if self.flag {
            &mut self.buffer
        } else {
            &mut self.buffer1
        }
    }
}

impl LogAppender for BufferAppender {
    fn do_logs(&mut self, records: &[FastLogRecord]) {
        for x in records {
            self.get_buffer().push_str(&x.formated);
            match &x.command {
                Command::CommandRecord => {}
                Command::CommandExit => {
                    println!("{}", self.buffer_max_size);
                }
                Command::CommandFlush(_) => {
                    self.buffer.clear();
                }
            }
        }
    }
}
