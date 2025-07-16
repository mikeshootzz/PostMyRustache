# PostMyRustache

<p align="center">
  <img src="./imgs/logo.png" alt="PostMyRustache logo" width="200">
</p>

<p align="center">
  <strong>A Rust-Powered MySQL-to-PostgreSQL Translation Layer</strong>
</p>

<p align="center">
  Seamlessly run your MySQL applications on PostgreSQL without code changes
</p>

<p align="center">
  <a href="https://github.com/mikeshootzz/PostMyRustache/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-GNU%20GPL-blue" alt="GPL License">
  </a>
  <a href="https://github.com/mikeshootzz/PostMyRustache/stargazers">
    <img src="https://img.shields.io/github/stars/mikeshootzz/PostMyRustache" alt="GitHub Stars">
  </a>
</p>

## 🚀 Features

- **MySQL Protocol**: Accepts MySQL connections and translates queries to PostgreSQL
- **Authentication**: Configurable MySQL authentication
- **High Performance**: Built with Rust and async Tokio
- **Docker Ready**: Easy deployment with Docker Compose
- **Query Translation**: Handles MySQL-specific syntax differences

## 🎯 Quick Start

### Using Docker Compose (Recommended)

```bash
git clone https://github.com/mikeshootzz/PostMyRustache.git
cd PostMyRustache
docker-compose up -d

# Connect with any MySQL client
mysql -h localhost -P 3306 -u admin -p
# Password: password
```

### Manual Installation

```bash
git clone https://github.com/mikeshootzz/PostMyRustache.git
cd PostMyRustache

# Configure environment
cp .env.example .env
# Edit .env with your settings

# Build and run
cargo run --release
```

## ⚙️ Configuration

Create a `.env` file in the project root:

```env
# PostgreSQL Connection
DB_HOST=localhost
DB_USER=postgres
DB_PASSWORD=your_postgres_password

# MySQL Authentication
MYSQL_USERNAME=admin
MYSQL_PASSWORD=password

# Optional
BIND_ADDRESS=0.0.0.0:3306
RUST_LOG=info
```

## 🔧 Usage

Connect using any MySQL client:

```bash
mysql -h localhost -P 3306 -u admin -p
```

Example queries:
```sql
CREATE DATABASE myapp;
USE myapp;

CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(255)
);

INSERT INTO users (name, email) VALUES ('John Doe', 'john@example.com');
SELECT * FROM users;
```

## 🛠️ Development

```bash
# Build
cargo build

# Run with logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Format code
cargo fmt
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

---

<p align="center">
  <strong>Made with ❤️ and 🦀 Rust</strong>
</p>