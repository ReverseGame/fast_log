use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::appender::{Command, FastLogRecord, LogAppender};
use crate::consts::LogSize;
use crate::error::LogError;

pub static END_SIGNAL: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

/// only write append into file
pub struct BufferAppender {
    buffer: HashMap<String, RefCell<String>>,
    log_size: LogSize,
}

impl BufferAppender {
    pub fn new(log_size: LogSize) -> Result<BufferAppender, LogError> {
        Ok(Self {
            buffer: HashMap::new(),
            log_size,
        })
    }

    pub fn get_buffer(&mut self, key: &str) -> &mut String {
        let data = self.buffer.entry(key.to_string()).or_insert_with(|| {
            RefCell::new(String::with_capacity(self.log_size.len() + 1024 * 10))
        });
        data.get_mut()
    }
}

impl LogAppender for BufferAppender {
    fn do_logs(&mut self, records: &[FastLogRecord]) {
        for x in records {
            self.get_buffer(&x.module_path).push_str(&x.formated);
            match &x.command {
                Command::CommandRecord => {}
                Command::CommandExit => {
                    for (_, v) in self.buffer.iter() {
                        let data = v.take();
                        println!("{}", data.len());
                    }
                    END_SIGNAL.store(true, Ordering::Relaxed);
                }
                Command::CommandFlush(_) => {}
            }
        }
        let mut keys = vec![];
        for (key, v) in self.buffer.iter() {
            if v.borrow().len() > self.log_size.len() {
                let data = v.take();
                println!("{}", data.len());
                keys.push(key.clone());
            }
        }
        for key in keys {
            self.buffer.insert(
                key.to_string(),
                RefCell::new(String::with_capacity(self.log_size.len() + 1024 * 10)),
            );
        }
    }
}
