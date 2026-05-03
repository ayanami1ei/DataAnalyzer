use std::{
    fs::OpenOptions,
    io::Write,
    sync::{Mutex, OnceLock},
};

const LOG_FILE_PATH: &str = "log.txt";

fn log_file_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn append_log_line(log: &str) {
    let _guard = match log_file_lock().lock() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("[log] lock poisoned: {}", e);
            return;
        }
    };

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_PATH)
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", log) {
                eprintln!("[log] write failed: {}", e);
            }
        }
        Err(e) => {
            eprintln!("[log] open failed: {}", e);
        }
    }
}

pub fn append_log_lines(logs: &[String]) {
    if logs.is_empty() {
        return;
    }

    let _guard = match log_file_lock().lock() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("[log] lock poisoned: {}", e);
            return;
        }
    };

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_PATH)
    {
        Ok(mut file) => {
            for log in logs {
                if let Err(e) = writeln!(file, "{}", log) {
                    eprintln!("[log] write failed: {}", e);
                    return;
                }
            }
        }
        Err(e) => {
            eprintln!("[log] open failed: {}", e);
        }
    }
}

pub trait LogSender {
    fn send_log(&self, log: &str) {
        append_log_line(log);
    }

    fn send_logs(&self, logs: &[String]) {
        append_log_lines(logs);
    }
}
