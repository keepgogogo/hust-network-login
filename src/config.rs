use std::env;
use std::fs;

pub struct Config {
    pub username: String,
    pub password: String,
    pub campus_dns: Option<String>,
}

impl Config {
    fn validate_and_assemble(
        username: Option<&str>,
        password: Option<&str>,
        campus_dns: Option<&str>,
    ) -> Result<Self, &'static str> {
        match (username, password) {
            (Some(_), None) => Err("missing password"),
            (None, Some(_)) => Err("missing username"),
            (None, None) => Err("missing username and password"),
            (Some(username), Some(password)) => Ok(Self {
                username: username.to_owned(),
                password: password.to_owned(),
                campus_dns: campus_dns.map(|s| s.to_owned()),
            }),
        }
    }

    pub fn from_env(verbose: bool) -> Option<Self> {
        if verbose {
            println!("reading configuration from environment variables");
        }
        let username = match env::var("HUST_NETWORK_LOGIN_USERNAME") {
            Ok(v) => Some(v),
            Err(_) => None,
        };
        let password = match env::var("HUST_NETWORK_LOGIN_PASSWORD") {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        let result = match Self::validate_and_assemble(
            username.as_ref().map(String::as_str),
            password.as_ref().map(String::as_str),
            None,
        ) {
            Ok(cfg) => cfg,
            Err(_) => return None,
        };

        Some(result)
    }

    pub fn from_file(path: &str, verbose: bool) -> Option<Self> {
        if verbose {
            println!("reading configuration from file: {path}");
        }

        let raw = match fs::read(&path) {
            Ok(data) => data,
            Err(_) => return None,
        };

        let configuration = match String::from_utf8(raw) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let mut lines = configuration.lines();
        let username = lines.next();
        let password = lines.next();
        let campus_dns = lines.next().filter(|s| !s.trim().is_empty());

        let result = match Self::validate_and_assemble(username, password, campus_dns) {
            Ok(cfg) => cfg,
            Err(_) => return None,
        };

        Some(result)
    }

    pub fn from_args() -> Option<Self> {
        let args: Vec<String> = env::args().collect();
        let mut config_path: Option<String> = None;
        let mut cli_dns: Option<String> = None;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "--dns" => {
                    if i + 1 < args.len() {
                        cli_dns = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        println!("Error: --dns requires an IP address argument");
                        std::process::exit(1);
                    }
                }
                arg if !arg.starts_with('-') => {
                    config_path = Some(arg.to_string());
                    i += 1;
                }
                _ => {
                    println!("Unknown argument: {}", args[i]);
                    i += 1;
                }
            }
        }

        let mut config = if let Some(path) = config_path {
            Self::from_file(&path, true)
        } else {
            None
        };

        if let Some(ref mut cfg) = config {
            if let Some(dns) = cli_dns {
                cfg.campus_dns = Some(dns);
            }
        } else if let Some(dns) = cli_dns {
            let env_config = Self::from_env(true);
            if let Some(mut cfg) = env_config {
                cfg.campus_dns = Some(dns);
                config = Some(cfg);
            }
        }

        config
    }
}

fn print_help() {
    let default_paths = if cfg!(windows) {
        r#"                      1. 命令行指定的路径 / CLI specified path
                      2. %APPDATA%\hust-network-login\config
                      3. 环境变量 / Environment variables"#
    } else {
        r#"                      1. 命令行指定的路径 / CLI specified path
                      2. /etc/hust-network-login.conf
                      3. /etc/hust-network-login/config
                      4. 环境变量 / Environment variables"#
    };

    println!(
        r#"hust-network-login - 华中科技大学校园网自动登录工具
HUST Campus Network Auto-Login Tool

用法 / Usage:
  hust-network-login [选项] [配置文件路径]
  hust-network-login [options] [config_file_path]

选项 / Options:
  -h, --help          显示此帮助信息并退出
                      Show this help message and exit
  --dns <IP>          指定校园网 DNS 服务器地址（用于绕过第三方 DNS）
                      Specify campus DNS server (to bypass third-party DNS)

参数 / Arguments:
  配置文件路径         可选，默认按以下顺序查找：
  config_file_path    Optional, searched in the following order:
{}
配置文件格式 / Config File Format:
  第一行: 用户名 / Line 1: Username
  第二行: 密码   / Line 2: Password
  第三行: DNS IP (可选) / Line 3: DNS IP (optional)

环境变量 / Environment Variables:
  HUST_NETWORK_LOGIN_USERNAME  用户名 / Username
  HUST_NETWORK_LOGIN_PASSWORD  密码   / Password

示例 / Examples:
  hust-network-login
  hust-network-login --dns 10.0.0.1
  hust-network-login /path/to/config.conf
  hust-network-login --dns 10.0.0.1 /path/to/config.conf
"#,
        default_paths
    );
}

pub fn print_help_and_exit() {
    print_help();
    std::process::exit(1);
}
