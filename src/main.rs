mod config;
mod dns;
mod encrypt;

use std::env;
use std::io;
use std::thread;
use std::time::Duration;

use config::Config;
use dns::resolve_via_dns;

const PROBE_DOMAIN: &str = "www.baidu.com";

fn extract<'a>(text: &'a str, prefix: &'a str, suffix: &'a str) -> io::Result<&'a str> {
    let left = text.find(prefix);
    let right = text.find(suffix);
    if let (Some(l), Some(r)) = (left, right) {
        if l + prefix.len() < r {
            return Ok(&text[l + prefix.len()..r]);
        }
    }
    Err(io::ErrorKind::InvalidData.into())
}

fn login(username: &str, password: &str, campus_dns: Option<&str>) -> io::Result<()> {
    let target_ip = if let Some(dns) = campus_dns {
        match resolve_via_dns(PROBE_DOMAIN, dns) {
            Ok(ip) => {
                println!("resolved {} via DNS {}: {}", PROBE_DOMAIN, dns, ip);
                ip
            }
            Err(e) => {
                println!("Warning: DNS resolution failed: {}, falling back to system DNS", e);
                PROBE_DOMAIN.to_string()
            }
        }
    } else {
        PROBE_DOMAIN.to_string()
    };

    let probe_url = format!("http://{}/", target_ip);

    let resp = minreq::get(&probe_url)
        .with_header("Host", PROBE_DOMAIN)
        .with_header("User-Agent", "Mozilla/5.0")
        .with_timeout(10)
        .send()
        .map_err(|e| {
            println!("probe request failed: {}", e);
            io::ErrorKind::ConnectionRefused
        })?;
    let resp = resp.as_str().map_err(|e| {
        println!("invalid resp format {}", e);
        io::ErrorKind::InvalidData
    })?;

    if !resp.contains("/eportal/index.jsp")
        && !resp.contains("<script>top.self.location.href='http://")
    {
        return Ok(());
    }

    let portal_ip = extract(
        resp,
        "<script>top.self.location.href='http://",
        "/eportal/index.jsp",
    )?;
    println!("portal ip: {}", portal_ip);

    let mac = extract(resp, "mac=", "&t=")?;
    println!("mac: {}", mac);

    let encrypt_pass = encrypt::encrypt_pass(format!("{}>{}", password, mac));

    let query_string = extract(resp, "/eportal/index.jsp?", "'</script>\r\n")?;
    println!("query_string: {}", query_string);

    let query_string = urlencoding::encode(query_string);

    let body = format!(
        "userId={}&password={}&service=&queryString={}&passwordEncrypt=true",
        username, encrypt_pass, query_string
    );

    let login_url = format!("http://{}/eportal/InterFace.do?method=login", portal_ip);

    let resp = minreq::post(login_url)
        .with_body(body)
        .with_header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .with_header("Accept", "*/*")
        .with_header("User-Agent", "hust-network-login")
        .with_timeout(10)
        .send()
        .map_err(|e| {
            println!("portal boom! {}", e);
            io::ErrorKind::ConnectionRefused
        })?;

    let resp = resp.as_str().map_err(|e| {
        println!("invalid login resp format {}", e);
        io::ErrorKind::InvalidData
    })?;

    println!("login resp: {}", resp);

    if resp.contains("success") {
        Ok(())
    } else {
        Err(io::ErrorKind::PermissionDenied.into())
    }
}

#[test]
fn login_test() {
    let _ = login("username", "password", None);
}

fn main() {
    let config = Config::from_args()
        .or_else(|| Config::from_env(false))
        .or_else(|| Config::from_file("/etc/hust-network-login.conf", false))
        .or_else(|| Config::from_file("/etc/hust-network-login/config", false))
        .or_else(|| {
            if cfg!(windows) {
                let appdata = env::var("APPDATA").ok()?;
                Config::from_file(&format!("{}\\hust-network-login\\config", appdata), false)
            } else {
                None
            }
        });

    match config {
        Some(cfg) => {
            println!(
                "starting with DNS: {}",
                cfg.campus_dns.as_deref().unwrap_or("system default")
            );

            loop {
                match login(&cfg.username, &cfg.password, cfg.campus_dns.as_deref()) {
                    Ok(_) => {
                        println!("login ok. awaiting...");
                        thread::sleep(Duration::from_secs(15));
                    }
                    Err(e) => {
                        println!("error! {}", e);
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        }
        None => {
            config::print_help_and_exit();
        }
    }
}
