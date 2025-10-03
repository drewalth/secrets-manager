# Secrets Manager

A secure local secrets manager for development environments. Store and manage your project's environment variables with strong encryption.

### Why

Managing secrets for development environments can be a pain. This tool is designed to help you manage your secrets in an easy way without relying on third party services.

## Features

- 🔐 **AES-256-GCM Encryption**: Your secrets are encrypted with industry-standard encryption
- 📁 **Project-based Organization**: Group secrets by project name
- 🚀 **Multiple Export Formats**: Export to shell variables, .env files, or JSON
- 🛡️ **Password Protection**: Master password required for all operations
- 💻 **CLI Interface**: Easy-to-use command-line interface
- 🔒 **Local Storage**: All data stored **locally** in your home directory

## Installation

### From Source

```bash
git clone https://github.com/drewalth/secrets-manager.git
cd secrets-manager
cargo build --release
sudo cp target/release/secrets-manager /usr/local/bin/
```

## Usage

> Tip: for brevity, create an alias for the secrets-manager command. In your `.bashrc` or `.zshrc` file, add:
> ```bash
> alias secrets="secrets-manager"
> ```

### Create a New Project

```bash
secrets-manager create my-project
```

This will prompt you to set a master password for the project.

### List All Projects

```bash
secrets-manager list
```

### Add Secrets to a Project

```bash
# Add a secret (will prompt for value)
secrets-manager add my-project API_KEY

# Add a secret with value
secrets-manager add my-project DATABASE_URL "postgres://localhost:5432/mydb"
```

### View Project Secrets

```bash
secrets-manager show my-project
```

### Export Secrets

```bash
# Export as shell variables (default)
secrets-manager export my-project

# Export to .env file
secrets-manager export my-project --format env --output .env

# Export as JSON
secrets-manager export my-project --format json
```

### Remove Secrets

```bash
secrets-manager remove my-project API_KEY
```

### Delete a Project

```bash
secrets-manager delete my-project
```

## Security

- **Encryption**: All data is encrypted using AES-256-GCM
- **Key Derivation**: Passwords are strengthened using PBKDF2 with 100,000 iterations
- **Salt & IV**: Each encryption operation uses unique salt and initialization vector
- **Local Storage**: Data is stored in `~/.secrets_manager/` directory

## File Structure

```
~/.secrets_manager/
├── project1.encrypted
├── project2.encrypted
└── ...
```

Each project is stored as an encrypted JSON file containing:
- Project metadata (name, timestamps)
- Encrypted secrets
- Salt and nonce for decryption

## Examples

### Setting up a new project

```bash
# Create project
secrets-manager create my-api

# Add secrets
secrets-manager add my-api API_KEY "sk-1234567890"
secrets-manager add my-api DATABASE_URL "postgres://user:pass@localhost/db"
secrets-manager add my-api REDIS_URL "redis://localhost:6379"

# Export to .env file
secrets-manager export my-api --format env --output .env
```

### Loading secrets into your shell

```bash
# Export and source
secrets-manager export my-api > /tmp/secrets.sh
source /tmp/secrets.sh
```

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running the CLI

```bash
cargo run -- create my-project
```

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Security Notice

**This tool is designed for local development use only**. Do not use it for production secrets or sensitive data that requires enterprise-grade security solutions.

The author is not a security expert. Use at your own risk. If you find a security vulnerability, please report it to the author via GitHub issues.

The author, contributors, and maintainers are not responsible for any damage or loss caused by the use of this tool.

If you're using this tool for commercial use, adhere to your company's security policies. You are responsible for your own security.
If your company requires you to use a third-party secrets manager, use that instead.

## Third-party Secrets Managers

For production use, consider using a third-party secrets manager like [Vault](https://www.vaultproject.io/) or [AWS Secrets Manager](https://aws.amazon.com/secrets-manager/).