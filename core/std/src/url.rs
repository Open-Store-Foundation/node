use url::Url;

pub trait Localhost {
    fn is_localhost(&self) -> bool;
}

impl Localhost for Url {
    fn is_localhost(&self) -> bool {
        if let Some(host) = self.host_str() {
            if host.ends_with("localhost") {
                return true;
            }

            if host.ends_with("127.0.0.1") {
                return true;
            }

            if host.starts_with("192.168.") {
                return true;
            }
        }
        
        return false;
    }
}