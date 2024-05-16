# Komprender - Tauri & Rust

## Introduction
Komprender is a desktop application designed to interact with Kafka clusters. Built with Tauri and Rust, it aims to provide developers with a powerful yet user-friendly interface for local Kafka development. While it has the potential for production use, it was primarily developed for educational purposes and local development environments.

## Features
- **Topic Inspection**: Browse, create, and manage topics within your clusters.
- **Message Publishing & Consumption**: Produce messages to topics and consume them with a user-friendly interface.
- **Avro producing**: Produce message within schema registry based on avro schemas.
## Getting Started
### Prerequisites
- Ensure you have Rust installed on your machine.
- Kafka cluster accessible from your local development environment.

### Installation
1. Clone the repository: `git clone git@github.com:aleksandr-gorokhov/komprender.git`
2. Install deps: `yarn install`
3. To start the application, run: `yarn tauri build`

## Known Limitations
- Authentication is not supported.
- Performance in large-scale production environments has not been extensively tested.

## Disclaimer
This application was developed as a learning project and, while it offers many useful features for Kafka development, it comes with no guarantees for stability or performance in production environments. Please use it at your own risk and conduct thorough testing before considering it for production use. The author(s) are not responsible for any issues arising from its use in production environments.

## Contributing
Contributions are welcome! If you have suggestions for improvements, or if you've found a bug and want to report it, please open an issue or submit a pull request.

## License
This project is licensed under the [MIT License](LICENSE).
