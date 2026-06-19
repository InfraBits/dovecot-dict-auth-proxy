use tempfile::TempDir;

use super::super::mailbox_lookup;
use super::{make_config, make_mailbox};

#[test]
fn valid_mailbox() {
    let dir = TempDir::new().unwrap();
    make_mailbox(&dir, "alice@example.com", "hunter2\n");
    let info = mailbox_lookup("alice@example.com", &make_config(&dir)).unwrap();
    assert_eq!(info.email, "alice@example.com");
    assert_eq!(info.password, "hunter2");
    assert!(info.mail.ends_with("/Maildir"));
    assert!(info.home.contains("alice"));
}

#[test]
fn no_at_sign() {
    let dir = TempDir::new().unwrap();
    assert!(mailbox_lookup("notanemail", &make_config(&dir)).is_none());
}

#[test]
fn missing_mailbox() {
    let dir = TempDir::new().unwrap();
    assert!(mailbox_lookup("ghost@example.com", &make_config(&dir)).is_none());
}

#[test]
fn dotdot_local_rejected() {
    let dir = TempDir::new().unwrap();
    assert!(mailbox_lookup("..@example.com", &make_config(&dir)).is_none());
}

#[test]
fn dotdot_domain_rejected() {
    let dir = TempDir::new().unwrap();
    assert!(mailbox_lookup("alice@..", &make_config(&dir)).is_none());
}

#[test]
fn slash_in_local_rejected() {
    let dir = TempDir::new().unwrap();
    assert!(mailbox_lookup("a/b@example.com", &make_config(&dir)).is_none());
}

#[test]
fn password_whitespace_trimmed() {
    let dir = TempDir::new().unwrap();
    make_mailbox(&dir, "bob@example.com", "  trimmed  \n");
    let info = mailbox_lookup("bob@example.com", &make_config(&dir)).unwrap();
    assert_eq!(info.password, "trimmed");
}
