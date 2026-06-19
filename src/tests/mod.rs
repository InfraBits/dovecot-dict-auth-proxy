use std::fs;
use std::io::Cursor;

use tempfile::TempDir;

use super::models::Config;
use super::handle_client;

mod lookup;
mod protocol;

pub fn make_config(dir: &TempDir) -> Config {
    Config {
        path_template: format!("{}/{{domain}}/mailboxes/{{user}}", dir.path().display()),
    }
}

pub fn make_mailbox(dir: &TempDir, email: &str, password: &str) {
    let (local, domain) = email.split_once('@').unwrap();
    let mailbox = dir.path().join(domain).join("mailboxes").join(local);
    fs::create_dir_all(&mailbox).unwrap();
    fs::write(mailbox.join("password"), password).unwrap();
}

pub fn run(input: &str, config: &Config) -> String {
    let mut output = Vec::new();
    handle_client(Cursor::new(input.as_bytes()), &mut output, config);
    String::from_utf8(output).unwrap()
}
