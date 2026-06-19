use serde::Serialize;

#[derive(Clone)]
pub struct Config {
    pub path_template: String,
}

pub struct Mailbox {
    pub email: String,
    pub home: String,
    pub uid: u32,
    pub gid: u32,
    pub mail: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct PassdbResponse {
    pub password: String,
    pub userdb_user: String,
    pub userdb_home: String,
    pub userdb_uid: u32,
    pub userdb_gid: u32,
    pub userdb_mail: String,
}

impl From<Mailbox> for PassdbResponse {
    fn from(m: Mailbox) -> Self {
        Self {
            password: m.password,
            userdb_user: m.email,
            userdb_home: m.home,
            userdb_uid: m.uid,
            userdb_gid: m.gid,
            userdb_mail: m.mail,
        }
    }
}

#[derive(Serialize)]
pub struct UserdbResponse {
    pub user: String,
    pub home: String,
    pub uid: u32,
    pub gid: u32,
    pub mail: String,
}

impl From<Mailbox> for UserdbResponse {
    fn from(m: Mailbox) -> Self {
        Self {
            user: m.email,
            home: m.home,
            uid: m.uid,
            gid: m.gid,
            mail: m.mail,
        }
    }
}
