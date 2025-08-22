use crate::appender::{Command, FastLogRecord};
use crate::config::Config;
use crate::error::LogError;
use crate::{chan, spawn, Receiver, SendError, Sender, WaitGroup};
use log::{LevelFilter, Log, Metadata, Record};
use std::sync::{Arc, LazyLock, OnceLock};
use std::time::SystemTime;
use dashmap::DashMap;
pub static LOGGERS: LazyLock<DashMap<String, &'static Logger>> = LazyLock::new(DashMap::new);
/// get Logger,but you must call `fast_log::init`
pub fn logger(key: &str) -> &'static Logger {
    let key = if key.contains("::") {
        key.split_once( "::").unwrap().0
    } else {
        key
    };
    if LOGGERS.contains_key(key) {
        LOGGERS.get(key).unwrap().value()
    } else {
        let _ = init(Config::new().chan_len(Some(5000)).file(&format!("{}.log", key)), key);
        LOGGERS.get(key).unwrap().value()
    }
}


pub struct Logger {
    pub cfg: OnceLock<Config>,
    pub send: OnceLock<Sender<FastLogRecord>>,
    pub recv: OnceLock<Receiver<FastLogRecord>>,
}

impl Logger {
    pub fn default() -> Self {
        Self {
            cfg: OnceLock::default(),
            send: OnceLock::default(),
            recv: OnceLock::default(),
        }
    }

    pub fn set_level(&self, level: LevelFilter) {
        log::set_max_level(level);
    }

    pub fn get_level(&self) -> LevelFilter {
        log::max_level()
    }

    /// print no other info
    pub fn print(&self, log: String) -> Result<(), SendError<FastLogRecord>> {
        let fast_log_record = FastLogRecord {
            command: Command::CommandRecord,
            level: log::Level::Info,
            target: "".to_string(),
            args: "".to_string(),
            module_path: "".to_string(),
            file: "".to_string(),
            line: None,
            now: SystemTime::now(),
            formated: log,
        };
        let mut keys = vec![];
        for i in LOGGERS.iter() {
            keys.push(i.key().to_string());
        }
        for key in &keys {
            if let Some(send) = logger(key).send.get() {
                let _ = send.send(fast_log_record.clone());
            } else {
                // Ok(())
                println!("{}", crossbeam_channel::SendError(fast_log_record.clone()))
            }
        }
        Ok(())
    }
}

pub struct Loggers;

impl Loggers {
    pub fn set_level(&self, level: LevelFilter) {
        log::set_max_level(level);
    }

    pub fn get_level(&self) -> LevelFilter {
        log::max_level()
    }
}

impl Log for Loggers {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.get_level()
    }
    fn log(&self, record: &Record) {
        let key = record.module_path().unwrap_or("unknown");
        if let Some(filter) = logger(key).cfg.get() {
            if let Some(send) = logger(key).send.get() {
                for filter in filter.filters.iter() {
                    if !filter.do_log(record) {
                        return;
                    }
                }
                let _ = send.send(FastLogRecord {
                    command: Command::CommandRecord,
                    level: record.level(),
                    target: record.metadata().target().to_string(),
                    args: record.args().to_string(),
                    module_path: record.module_path().unwrap_or_default().to_string(),
                    file: record.file().unwrap_or_default().to_string(),
                    line: record.line().clone(),
                    now: SystemTime::now(),
                    formated: String::new(),
                });
            }
        }
    }
    fn flush(&self) {
        match flush() {
            Ok(v) => {
                v.wait();
            }
            Err(_) => {}
        }
    }
}

pub fn init(config: Config, key: &str) -> Result<&'static Logger, LogError> {
    if config.appends.is_empty() {
        return Err(LogError::from("[fast_log] appends can not be empty!"));
    }
    let (s, r) = chan(config.chan_len);
    let logger_default = Logger::default();
    logger_default
        .send
        .set(s)
        .map_err(|_| LogError::from("set fail"))?;
    logger_default
        .recv
        .set(r)
        .map_err(|_| LogError::from("set fail"))?;
    logger_default.set_level(config.level);
    logger_default
        .cfg
        .set(config)
        .map_err(|_| LogError::from("set fail="))?;
    LOGGERS.insert(key.to_string(), Box::leak(Box::new(logger_default)));

    let mut receiver_vec = vec![];
    let mut sender_vec: Vec<Sender<Arc<Vec<FastLogRecord>>>> = vec![];
    let cfg = logger(key).cfg.get().expect("logger cfg is none");
    for a in cfg.appends.iter() {
        let (s, r) = chan(cfg.chan_len);
        sender_vec.push(s);
        receiver_vec.push((r, a));
    }
    for (receiver, appender) in receiver_vec {
        spawn(move || {
            let mut exit = false;
            loop {
                let mut remain = vec![];
                if receiver.len() == 0 {
                    if let Ok(msg) = receiver.recv() {
                        remain.push(msg);
                    }
                }
                //recv all
                loop {
                    match receiver.try_recv() {
                        Ok(v) => {
                            remain.push(v);
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                //lock get appender
                let mut shared_appender = appender.lock();
                for msg in remain {
                    shared_appender.do_logs(msg.as_ref());
                    for x in msg.iter() {
                        match x.command {
                            Command::CommandRecord => {}
                            Command::CommandExit => {
                                exit = true;
                                continue;
                            }
                            Command::CommandFlush(_) => {
                                continue;
                            }
                        }
                    }
                }
                if exit {
                    break;
                }
            }
        });
    }
    let sender_vec = Arc::new(sender_vec);
    for _ in 0..1 {
        let key = key.to_string();
        let senders = sender_vec.clone();
        spawn(move || {
            loop {
                if let Some(recv) = logger(&key).recv.get() {
                    let mut remain = Vec::with_capacity(recv.len());
                    //recv
                    if recv.len() == 0 {
                        if let Ok(item) = recv.recv() {
                            remain.push(item);
                        }
                    }
                    //merge log
                    loop {
                        match recv.try_recv() {
                            Ok(v) => {
                                remain.push(v);
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                    let mut exit = false;
                    for x in &mut remain {
                        if x.formated.is_empty() {
                            logger(&key)
                                .cfg
                                .get()
                                .expect("logger cfg is none")
                                .format
                                .do_format(x);
                        }
                        if x.command.eq(&Command::CommandExit) {
                            exit = true;
                        }
                    }
                    let data = Arc::new(remain);
                    for x in senders.iter() {
                        let _ = x.send(data.clone());
                    }
                    if exit {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }
    Ok(logger(key))
}

pub fn exit() -> Result<(), LogError> {
    let fast_log_record = FastLogRecord {
        command: Command::CommandExit,
        level: log::Level::Info,
        target: String::new(),
        args: String::new(),
        module_path: String::new(),
        file: String::new(),
        line: None,
        now: SystemTime::now(),
        formated: String::new(),
    };
    let mut keys = vec![];
    for i in LOGGERS.iter() {
        keys.push(i.key().to_string());
    }
    for key in &keys {
        let result = logger(key)
            .send
            .get()
            .ok_or_else(|| LogError::from("not init"))?
            .send(fast_log_record.clone());
        match result {
            Ok(()) => {}
            _ => {
                println!("[fast_log] exit fail! key={:?}", key);
            }
        }
    }

    Ok(())
}

pub fn flush() -> Result<WaitGroup, LogError> {
    let wg = WaitGroup::new();
    let fast_log_record = FastLogRecord {
        command: Command::CommandFlush(wg.clone()),
        level: log::Level::Info,
        target: String::new(),
        args: String::new(),
        module_path: String::new(),
        file: String::new(),
        line: None,
        now: SystemTime::now(),
        formated: String::new(),
    };
        let mut keys = vec![];
        for i in LOGGERS.iter() {
            keys.push(i.key().to_string());
        }
    for key in &keys {
        let result = logger(key)
            .send
            .get()
            .ok_or_else(|| LogError::from("not init"))?
            .send(fast_log_record.clone());
        match result {
            Ok(()) => {}
            _ => {
                println!("[fast_log] flush fail! key={:?}", key);
            }
        }
    }

    Ok(wg)
}

pub fn print(log: String, key: &str) -> Result<(), SendError<FastLogRecord>> {
    logger(key).print(log)
}
