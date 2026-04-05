# HUST-Network-Login

极简主义的华中科技大学校园网络认证工具，支持有线和无线网络。下载即用，大小约为 400k，静态链接无依赖。为路由器等嵌入式设备开发，支持所有主流硬件软件平台。No Python, No Dependencies, No Bullshit.

## 使用

从 Release 下载对应硬件和操作系统平台的可执行文件。

### 配置文件

配置文件第一行为用户名，第二行为密码，第三行为可选的校园网 DNS（用于绕过第三方 DNS），例如

```text
M2020123123
mypasswordmypassword
10.0.0.1
```

保存为 my.conf

**配置文件查找顺序：**

| 平台 | 查找路径 |
|------|----------|
| Linux | 1. 命令行指定 2. `/etc/hust-network-login.conf` 3. `/etc/hust-network-login/config` 4. 环境变量 |
| Windows | 1. 命令行指定 2. `%APPDATA%\hust-network-login\config` 3. 环境变量 |

### 运行

```shell
# 使用配置文件
./hust-network-login ./my.conf

# 命令行指定 DNS（覆盖配置文件中的 DNS）
./hust-network-login --dns 10.0.0.1 ./my.conf

# 查看帮助
./hust-network-login --help
```

### DNS 配置说明

当你的设备（如路由器）配置了第三方 DNS（如 8.8.8.8 或 AdGuardHome）时，校园网网关的 DNS 欺骗机制会失效，导致自动登录无法触发。此时可以通过配置校园网 DNS 来解决：

1. 在配置文件第三行填写校园网 DNS 地址
2. 或使用 `--dns` 参数指定

程序会向指定的 DNS 发起 `www.baidu.com` 的解析请求，获取被校园网网关劫持的 IP，然后通过伪造 Host 头触发 HTTP 劫持完成登录。

### 环境变量

也支持通过环境变量配置：

```shell
export HUST_NETWORK_LOGIN_USERNAME=M2020123123
export HUST_NETWORK_LOGIN_PASSWORD=mypasswordmypassword
./hust-network-login
```

连接成功后，程序将会每间隔 15s 测试一次网络连通性。如果无法连接则进行重新登陆。

## 编译

### 本地编译

编译本地平台只需要使用 `cargo`。

```shell
cargo build --release
strip ./target/release/hust-network-login
```

strip 可以去除调试符号表，将体积减少到 500k 以下。

### 交叉编译

使用 `cross` 工具可以方便地进行交叉编译。以下以 Linux 作为编译机为例：

#### 安装 cross

```shell
cargo install cross
```

需要 Docker 环境，确保 Docker 已安装并运行。

#### Windows (x86_64)

Windows 有两种编译目标：**MSVC**（推荐）和 **GNU**。

| 特性 | MSVC (推荐) | GNU (MinGW) |
|------|-------------|-------------|
| 工具链 | Microsoft Visual C++ | MinGW-w64 (GCC) |
| 兼容性 | 最佳，原生 Windows | 可能有少量 Win32 API 兼容问题 |
| 体积 | 更小 (~260K) | 稍大 (~420K) |
| 依赖 | MSVC Runtime (系统自带) | 可能需要 MinGW DLL |

**MSVC 版本（推荐）**：使用 `cargo-xwin` 在 Linux 上交叉编译

```shell
# 安装 cargo-xwin
cargo install cargo-xwin

# 添加目标
rustup target add x86_64-pc-windows-msvc

# 编译（首次会自动下载 Windows CRT 和 SDK）
cargo xwin build --release --target x86_64-pc-windows-msvc
# 输出: target/x86_64-pc-windows-msvc/release/hust-network-login.exe
```

**GNU 版本**：使用 `mingw-w64` 交叉编译

```shell
# 安装 mingw-w64 (Ubuntu/Debian)
sudo apt-get install mingw-w64

# 添加目标并编译
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# 输出: target/x86_64-pc-windows-gnu/release/hust-network-login.exe
```

#### Linux (x86_64, 静态链接)

```shell
cross build --release --target x86_64-unknown-linux-musl
strip ./target/x86_64-unknown-linux-musl/release/hust-network-login
# 输出: target/x86_64-unknown-linux-musl/release/hust-network-login
```

#### Linux (ARM64)

```shell
cross build --release --target aarch64-unknown-linux-musl
aarch64-linux-gnu-strip ./target/aarch64-unknown-linux-musl/release/hust-network-login
# 输出: target/aarch64-unknown-linux-musl/release/hust-network-login
```

#### macOS (x86_64)

```shell
cross build --release --target x86_64-apple-darwin
# 输出: target/x86_64-apple-darwin/release/hust-network-login
```

#### macOS (ARM64/Apple Silicon)

```shell
cross build --release --target aarch64-apple-darwin
# 输出: target/aarch64-apple-darwin/release/hust-network-login
```

#### 路由器嵌入式平台

```shell
# MIPS (常见于老款路由器)
cross build --release --target mips-unknown-linux-musl
mips-linux-gnu-strip ./target/mips-unknown-linux-musl/release/hust-network-login

# MIPS Little Endian
cross build --release --target mipsel-unknown-linux-musl
mipsel-linux-gnu-strip ./target/mipsel-unknown-linux-musl/release/hust-network-login

# ARM (常见于新款路由器)
cross build --release --target arm-unknown-linux-musleabihf
arm-linux-gnueabihf-strip ./target/arm-unknown-linux-musleabihf/release/hust-network-login
```

更多支持的目标平台请参考 [cross 支持的目标列表](https://github.com/rust-embedded/cross)。
