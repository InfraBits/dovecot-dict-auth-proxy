use tempfile::TempDir;

use super::{make_config, make_mailbox, run};

#[test]
fn handshake_produces_no_output() {
    let dir = TempDir::new().unwrap();
    assert_eq!(run("H3\t1\tdovecot-dict-proxy\n", &make_config(&dir)), "");
}

#[test]
fn empty_line_ends_session() {
    let dir = TempDir::new().unwrap();
    assert_eq!(run("\nFsomething\n", &make_config(&dir)), "");
}

#[test]
fn passdb_found() {
    let dir = TempDir::new().unwrap();
    make_mailbox(&dir, "alice@example.com", "s3cr3t");
    let out = run("Lshared/passdb/alice@example.com\n", &make_config(&dir));
    assert!(out.starts_with('O'), "expected O response, got: {out}");
    let json: serde_json::Value =
        serde_json::from_str(out.trim_start_matches('O').trim_end()).unwrap();
    assert_eq!(json["password"], "s3cr3t");
    assert!(json.get("userdb_home").is_some());
    assert!(json.get("userdb_uid").is_some());
    assert!(json.get("userdb_gid").is_some());
    assert!(json["userdb_mail"].as_str().unwrap().contains("Maildir"));
}

#[test]
fn passdb_not_found() {
    let dir = TempDir::new().unwrap();
    assert_eq!(
        run("Lshared/passdb/ghost@example.com\n", &make_config(&dir)),
        "N\n"
    );
}

#[test]
fn userdb_found_no_password() {
    let dir = TempDir::new().unwrap();
    make_mailbox(&dir, "bob@example.com", "secret");
    let out = run("Lshared/userdb/bob@example.com\n", &make_config(&dir));
    assert!(out.starts_with('O'), "expected O response, got: {out}");
    let json: serde_json::Value =
        serde_json::from_str(out.trim_start_matches('O').trim_end()).unwrap();
    assert!(json.get("password").is_none(), "password must not appear in userdb response");
    assert!(json.get("home").is_some());
    assert!(json.get("uid").is_some());
    assert!(json.get("gid").is_some());
}

#[test]
fn userdb_not_found() {
    let dir = TempDir::new().unwrap();
    assert_eq!(
        run("Lshared/userdb/ghost@example.com\n", &make_config(&dir)),
        "N\n"
    );
}

#[test]
fn unknown_lookup_prefix_returns_f() {
    let dir = TempDir::new().unwrap();
    assert_eq!(run("Lshared/other/something\n", &make_config(&dir)), "F\n");
}

#[test]
fn unknown_command_returns_f() {
    let dir = TempDir::new().unwrap();
    assert_eq!(run("Xwhatever\n", &make_config(&dir)), "F\n");
}

#[test]
fn multiple_lookups_in_one_session() {
    let dir = TempDir::new().unwrap();
    make_mailbox(&dir, "alice@example.com", "pw");
    let input = "Lshared/passdb/alice@example.com\nLshared/passdb/ghost@example.com\n";
    let out = run(input, &make_config(&dir));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with('O'));
    assert_eq!(lines[1], "N");
}
