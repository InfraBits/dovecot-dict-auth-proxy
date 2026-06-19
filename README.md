# Dovecot authentication proxy

A Dovecot dict-proxy authentication service that resolves email credentials from per-user password files on the filesystem.

## How it works

Dovecot supports pluggable `passdb` and `userdb` backends via its [dict proxy protocol](https://doc.dovecot.org/configuration_manual/dict/).

When a user logs in, Dovecot opens a Unix socket and sends a lookup request.

This daemon answers those requests by reading a `password` file from a predictable directory path derived from the email address.

For `alice@example.com` with the default path template, the daemon will attempt to load:

- `/srv/example.com/mailboxes/alice` - uid/gid of user resolved
- `/srv/example.com/mailboxes/alice/password` - location of hashed password
- `/srv/example.com/mailboxes/alice/Maildir` - location of Maildir

The password file should contain a single hashed password in the scheme configured in Dovecot (default: `SHA256-CRYPT`).

The password can be generated with e.g. `doveadm pw -s SHA256-CRYPT`.

## Running

The binary must be started as root, where the socket will be bound and then privileges dropped.

```
dovecot-auth-proxy [--user dovecot] [--group dovecot] [--socket-path /run/dovecot/dict-auth] [--mailbox-path /srv/{domain}/mailboxes/{user}]
```

## Example Dovecot configuration

Configure the auth provider e.g. in `/etc/dovecot/dict-auth.conf`:

```
uri = proxy:/run/dovecot/dict-auth:
default_pass_scheme = SHA256-CRYPT
iterate_disable = yes

key passdb {
  key = passdb/%u
  format = json
}

key userdb {
  key = userdb/%u
  format = json
}

passdb_objects = passdb
userdb_objects = userdb
```

Then in `/etc/dovecot.conf` reference the auth provider e.g.
```
passdb {
  driver = dict
  args = /etc/dovecot/dict-auth.conf
}

userdb {
  driver = dict
  args = /etc/dovecot/dict-auth.conf
}
```
