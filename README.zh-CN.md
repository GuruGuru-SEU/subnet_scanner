# 子网扫描器与代理测试工具 (Subnet Scanner)

[![许可证: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一款使用 Rust 编写的高性能、异步、多线程的工具，用于扫描指定子网的开放端口，并将其作为 HTTP 代理进行连接测试。它还集成了 Geo-IP 地理位置查询和灵活的输入/输出选项。

本项目旨在实现极致的速度和优秀的用户体验，充分利用了 Rust 现代化生态的强大能力，例如使用 `rayon` 进行并行扫描，使用 `tokio` 进行异步测试。

## 主要特性

- **高性能扫描**: 使用 `rayon` 在所有可用的 CPU核心上并发扫描子网中的所有主机。
- **异步测试**: 使用 `tokio` 同时测试数百个潜在的代理服务器，进行连接和地理位置查询，全程无阻塞。
- **地理位置查询 (Geo-IP)**: 自动通过公共 API 检测可用代理的城市和国家信息。
- **灵活的输入源**: 支持扫描全新的子网，或从 CSV 文件中读取 IP 列表进行重新测试。
- **可配置的输出**:
    - **详细模式**: 查看实时的、彩色的日志，清晰展示每个端口的发现、成功、失败和地理位置查询结果。
    - **安静模式**: 从文件输入时显示简洁的进度条，扫描子网时显示一个旋转图标。
- **保存为 CSV**: 将所有可用的代理服务器列表（按响应速度排序）导出为 CSV 文件。
- **跨平台**: 可在 Windows、macOS 和 Linux 上编译和运行。

## 安装

### 环境要求

您需要安装 Rust 工具链。如果尚未安装，请访问 [rustup.rs](https://rustup.rs/) 获取。

### 安装步骤

1.  **克隆代码仓库:**
    ```bash
    git clone <your-repo-url>
    cd subnet_scanner
    ```

2.  **以 release 模式构建项目以获得最佳性能:**
    ```bash
    cargo build --release
    ```

3.  生成的可执行文件位于 `target/release/subnet_scanner`。您可以将其复制到系统的 `PATH` 路径下的目录中，方便全局调用。

## 使用方法

本工具提供了一系列命令行参数以实现灵活性。您可以随时使用 `--help` 查看所有可用选项。

```bash
cargo run --release -- --help
```

### 使用示例

#### 1. 扫描子网（默认安静模式，显示旋转图标）
此命令将扫描 `192.168.1.0/24` 网段中开放了 `8080` 端口的主机，并在运行时显示一个旋转图标。

```bash
cargo run --release -- --subnet 192.168.1.0/24 -p 8080
```

#### 2. 使用详细的彩色日志进行扫描
添加 `--verbose` (或 `-v`) 参数可以启用实时的详细日志输出。

```bash
cargo run --release -- --subnet 192.168.1.0/24 -p 8080 --verbose
```
**详细日志输出示例:**
```
⠋ Scanning subnet...
[FOUND]   Potential proxy at 192.168.1.55:8080
[FOUND]   Potential proxy at 192.168.1.101:8080
[FAIL]     192.168.1.101:8080: operation timed out
[SUCCESS] 192.168.1.55:8080 connected in 312ms
[GEO]      192.168.1.55:8080 located in Los Angeles, United States
```

#### 3. 扫描并将结果保存到 CSV 文件
使用 `--output` (或 `-o`) 参数可以将最终的表格结果保存到文件。

```bash
cargo run --release -- --subnet 192.168.1.0/24 --output working_proxies.csv
```

#### 4. 从 CSV 文件读取代理进行测试
如果您已有一个 IP 地址列表，可以重新对它们进行测试。此模式会跳过扫描阶段。首先，创建一个名为 `proxies.csv` 的文件，格式如下：

**`proxies.csv` 文件内容:**
```csv
"IP Address"
8.8.8.8
1.1.1.1
9.9.9.9
```

然后，使用 `--input` (或 `-i`) 参数运行工具。此时会显示一个进度条。

```bash
cargo run --release -- --input proxies.csv -p 80
```

### 最终报告示例

所有任务完成后，一个按响应速度（从快到慢）排序的总结表格会打印到控制台。

```
--- Final Results ---
+------+------------+---------------+--------------------------+
| Rank | IP Address | Response Time | Location                 |
+================================================================+
| 1    | 8.8.4.4    | 45 ms         | Mountain View, United States |
+------+------------+---------------+--------------------------+
| 2    | 1.1.1.1    | 58 ms         | Sydney, Australia        |
+------+------------+---------------+--------------------------+
```

## 许可证 (License)

本项目采用 MIT 许可证。