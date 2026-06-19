use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use clap::Parser;
use nix::unistd::{Group, User};

mod models;
use models::{Config, Mailbox, PassdbResponse, UserdbResponse};

#[derive(Parser)]
#[command(about = "Dovecot dict authentication service backed by per-user password files", version = env!("BUILD_VERSION"))]
struct Args {
    #[arg(long, default_value = "dovecot")]
    user: String,

    #[arg(long, default_value = "dovecot")]
    group: String,

    #[arg(long, default_value = "/run/dovecot/dict-auth")]
    socket_path: String,

    #[arg(long, default_value = "/srv/{domain}/mailboxes/{user}")]
    mailbox_path: String,
}

fn is_safe_component(s: &str) -> bool {
    let mut it = Path::new(s).components();
    matches!(it.next(), Some(std::path::Component::Normal(_)))
        && it.next().is_none()
        && !s.contains('\0')
}

fn mailbox_lookup(email: &str, config: &Config) -> Option<Mailbox> {
    let (local, domain) = email.split_once('@')?;

    if !is_safe_component(local) || !is_safe_component(domain) {
        eprintln!("unsafe path components in email: {email}");
        return None;
    }

    let home = PathBuf::from(
        config
            .path_template
            .replace("{domain}", domain)
            .replace("{user}", local),
    );

    let password_path = home.join("password");
    if !password_path.is_file() {
        eprintln!("no password file for {email}: {password_path:?}");
        return None;
    }

    let password = fs::read_to_string(&password_path).ok()?.trim().to_string();
    let meta = fs::metadata(&home).ok()?;
    let mail = format!("maildir:{}/Maildir", home.display());

    Some(Mailbox {
        email: email.to_string(),
        home: home.to_string_lossy().into_owned(),
        uid: meta.uid(),
        gid: meta.gid(),
        mail,
        password,
    })
}

fn ok_response(value: &impl serde::Serialize) -> String {
    let data = serde_json::to_string(value).expect("serialize");
    format!("O{data}\n")
}

fn dispatch(line: &str, config: &Config) -> Option<String> {
    if line.starts_with('H') {
        eprintln!("client handshake: {line}");
        return None;
    }

    if let Some(email) = line.strip_prefix("Lshared/passdb/") {
        eprintln!("passdb lookup: {email}");
        return Some(match mailbox_lookup(email, config) {
            None => "N\n".to_string(),
            Some(m) => {
                eprintln!("passdb match: {}", m.home);
                ok_response(&PassdbResponse::from(m))
            }
        });
    }

    if let Some(email) = line.strip_prefix("Lshared/userdb/") {
        eprintln!("userdb lookup: {email}");
        return Some(match mailbox_lookup(email, config) {
            None => "N\n".to_string(),
            Some(m) => {
                eprintln!("userdb match: {}", m.home);
                ok_response(&UserdbResponse::from(m))
            }
        });
    }

    eprintln!("unhandled command: {line}");
    Some("F\n".to_string())
}

fn handle_client<R: BufRead, W: Write>(reader: R, mut writer: W, config: &Config) {
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if line.is_empty() {
            break;
        }
        if let Some(response) = dispatch(&line, config) {
            let _ = writer.write_all(response.as_bytes());
            let _ = writer.flush();
        }
    }
}

#[cfg(test)]
mod tests;

fn main() {
    let args = Args::parse();

    let user = User::from_name(&args.user)
        .expect("getpwnam failed")
        .unwrap_or_else(|| panic!("user '{}' not found", args.user));
    let group = Group::from_name(&args.group)
        .expect("getgrnam failed")
        .unwrap_or_else(|| panic!("group '{}' not found", args.group));

    let socket_path = args.socket_path.as_str();
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path).expect("remove existing socket");
    }

    let listener = UnixListener::bind(socket_path).expect("bind unix socket");
    eprintln!("listening on {socket_path}");

    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o600))
        .expect("chmod socket");
    nix::unistd::chown(socket_path, Some(user.uid), Some(group.gid))
        .expect("chown socket");

    if nix::unistd::getuid().is_root() {
        #[cfg(target_os = "linux")]
        nix::unistd::setgroups(&[group.gid]).expect("setgroups");
        nix::unistd::setgid(group.gid).expect("setgid");
        nix::unistd::setuid(user.uid).expect("setuid");
    }

    nix::sys::stat::umask(nix::sys::stat::Mode::from_bits_truncate(0o027));

    let config = Arc::new(Config { path_template: args.mailbox_path });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer = stream.try_clone().expect("clone stream");
                let config = Arc::clone(&config);
                thread::spawn(move || {
                    handle_client(BufReader::new(stream), BufWriter::new(peer), &config);
                });
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }

    let _ = fs::remove_file(socket_path);
}
