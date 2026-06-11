use regex::Regex;
use std::sync::OnceLock;

pub(super) fn re_join() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"! Joining game '([0-9a-f\-]+)' place (\d+) at ([0-9\.]+)").unwrap()
    })
}

pub(super) fn re_joined() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"serverId: ([0-9\.]+)\|").unwrap())
}

pub(super) fn re_leave() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"Time to disconnect replication data").unwrap())
}

pub(super) fn re_udmux() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"UDMUX Address = ([0-9\.]+), Port = [0-9]+ \| RCC Server Address = ([0-9\.]+), Port = [0-9]+").unwrap()
    })
}

pub(super) fn re_bloxstrap_rpc() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\[BloxstrapRPC\] (.+)").unwrap())
}

pub(super) fn re_private_server_access_code() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#""accessCode":"([0-9a-f\-]{36})""#).unwrap())
}
