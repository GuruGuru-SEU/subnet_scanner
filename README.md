# Subnet Scanner & Proxy Tester

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, asynchronous, and multi-threaded tool written in Rust to scan subnets for open ports and test them as HTTP proxies. It includes Geo-IP location lookups and flexible input/output options.

This tool is designed to be both fast and user-friendly, leveraging the power of Rust's modern ecosystem with crates like `rayon` for parallel scanning and `tokio` for asynchronous testing.

## Key Features

- **High-Performance Scanning**: Uses `rayon` to scan all hosts in a subnet concurrently across all available CPU cores.
- **Asynchronous Testing**: Uses `tokio` to test hundreds of potential proxies simultaneously for connectivity and Geo-IP location without blocking.
- **Geo-IP Location**: Automatically detects the city and country of working proxies using a public API.
- **Flexible Input**: Scan a new subnet or re-test proxies from a CSV file.
- **Configurable Output**:
  - **Verbose Mode**: See real-time, colorful logs for every found port, success, failure, and geo-lookup.
  - **Quiet Mode**: A clean progress bar for file inputs or a simple spinner for subnet scans.
- **Save to CSV**: Export the list of working proxies, sorted by speed, to a CSV file.
- **Cross-Platform**: Compiles and runs on Windows, macOS, and Linux.

## Installation

### Prerequisites

You need to have the Rust toolchain installed. If you don't have it, you can get it from [rustup.rs](https://rustup.rs/).

### Steps

1.  **Clone the repository:**

    ```bash
    git clone <your-repo-url>
    cd subnet_scanner
    ```

2.  **Build the project in release mode for maximum performance:**

    ```bash
    cargo build --release
    ```

3.  The executable will be available at `target/release/subnet_scanner`. You can copy it to a directory in your system's `PATH` for easy access.

## Usage

The tool offers a range of command-line arguments for flexibility. You can always see the full list with `--help`.

```bash
cargo run --release -- --help
```

### Examples

#### 1. Scan a Subnet with a Spinner (Default Quiet Mode)

This will scan the `192.168.1.0/24` network for open port `8080` and show a spinner while running.

```bash
cargo run --release -- --subnet 192.168.1.0/24 -p 8080
```

#### 2. Scan a Subnet with Verbose, Colorful Logs

The `--verbose` (or `-v`) flag enables detailed real-time logging.

```bash
cargo run --release -- --subnet 192.168.1.0/24 -p 8080 --verbose
```

**Example Verbose Output:**

```
⠋ Scanning subnet...
[FOUND]   Potential proxy at 192.168.1.55:8080
[FOUND]   Potential proxy at 192.168.1.101:8080
[FAIL]     192.168.1.101:8080: operation timed out
[SUCCESS] 192.168.1.55:8080 connected in 312ms
[GEO]      192.168.1.55:8080 located in Los Angeles, United States
```

#### 3. Scan and Save Results to a CSV File

Use the `--output` (or `-o`) flag to save the final table to a file.

```bash
cargo run --release -- --subnet 192.168.1.0/24 --output working_proxies.csv
```

#### 4. Test Proxies from a CSV File

If you have a list of IPs, you can re-test them. This skips the scanning phase. Create a file named `proxies.csv` with the following format:

**`proxies.csv`:**

```csv
"IP Address"
8.8.8.8
1.1.1.1
9.9.9.9
```

Then run the tool with the `--input` (or `-i`) flag. A progress bar will be shown.

```bash
cargo run --release -- --input proxies.csv -p 80
```

### Final Report Example

After all tasks are complete, a summary table is printed to the console, sorted by the fastest response time.

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

## License

This project is licensed under the MIT License.