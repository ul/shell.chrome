use dirs;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::{exit, Command};

// those pervasive #[serde(default)] show that you might want separate structures
// for what you get from read and for what you put in send
#[derive(Serialize, Deserialize)]
struct Message {
    command: String,
    #[serde(default)]
    stdin: String,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
    #[serde(default)]
    status: i32,
    // just in case there some extra fields to be passed through
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

fn main() {
    loop {
        let mut message: Message =
            serde_json::from_str(&read()).expect("Failed to deserialize message");
        let command = &message.command;
        let home = dirs::home_dir().expect("Home directory is not found");
        let output = Command::new(command)
            .current_dir(home)
            .output()
            .expect("Failed to execute process");
        message.stdin = String::from(""); // you have condition here, but it looks like stdin for a child command will be always empty
        message.stdout =
            String::from_utf8(output.stdout).expect("Process stdout contains not valid UTF-8");
        message.stderr =
            String::from_utf8(output.stderr).expect("Process stderr contains not valid UTF-8");
        message.status = output.status.code().unwrap_or_default();
        send(&serde_json::to_string(&message).expect("Failed to serialize message"));
    }
}

fn read() -> String {
    let mut stdin = BufReader::new(std::io::stdin());
    let mut len_as_bytes: [u8; 4] = [0; 4];
    if let Err(_) = stdin.read_exact(&mut len_as_bytes) {
        exit(0);
    }
    let text_length_bytes = i32::from_ne_bytes(len_as_bytes);
    let mut reader = stdin.take(text_length_bytes as _);
    let mut result = String::new();
    reader
        .read_to_string(&mut result)
        .expect("Failed to read message");
    result
}

fn send(message: &str) {
    let mut stdout = BufWriter::new(std::io::stdout());
    let mut len_as_bytes: [u8; 4] = (message.len() as u32).to_ne_bytes();
    stdout
        .write_all(&mut len_as_bytes)
        .expect("Failed to write message length");
    stdout
        .write_all(message.as_bytes())
        .expect("Failed to write message");
}
