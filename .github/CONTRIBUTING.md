# Contributing to Agency

Thank you for your interest in contributing to **agency**! We welcome contributions from everyone. By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md) (if applicable) and the terms of our [CLA](../CLA.md).

## 📋 Legal Requirement

**All contributors must sign the Contributor License Agreement (CLA).**
When you open a Pull Request, a bot will check if you have signed. If not, you will need to post a comment saying:
> "I have read and agree to the CLA"

## 🛠 Development Setup

This project uses **Rust**. Ensure you have the latest stable toolchain installed.

1.  **Fork and Clone** the repository.
2.  **Install Dependencies & Setup**:
    ```bash
    make setup
    ```
    This command builds the project and sets up necessary ONNX runtimes.

## 🧪 Running Tests

We prioritize quality and stability. Please run the full test suite before submitting:

```bash
make test
```

This runs:
- Comprehensive Feature Tests
- Architecture Tests
- Unit Tests

## 🚀 Pull Request Process

1.  Create a new branch for your feature (`git checkout -b feature/amazing-feature`).
2.  Commit your changes (`git commit -m 'feat: add amazing feature'`).
3.  Push to the branch (`git push origin feature/amazing-feature`).
4.  Open a Pull Request.
5.  **Ensure all CI checks pass**, including the CLA check.

## 🐛 Reporting Bugs

Please use the [Bug Report Template](../.github/ISSUE_TEMPLATE/bug_report.md) when reporting issues. Include logs, reproduction steps, and environment details.
