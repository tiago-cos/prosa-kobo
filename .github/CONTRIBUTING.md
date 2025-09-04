# Contributing to Prosa-Kobo

Thank you for taking the time to contribute to Prosa-Kobo.  
Whether it’s reporting a bug, suggesting a feature, or submitting code, all contributions are welcome.

## How to Contribute

### Issues

- Anyone is free to open an issue.
- Please describe the problem or suggestion as clearly as possible.
- Screenshots, logs, or examples are appreciated if they help illustrate the issue.

### Pull Requests

Pull requests are welcome. Please make sure to follow these guidelines:

1. **Tests**
   - Ensure all tests pass. Instructions are in the [README](./README.md).
   - Add tests that demonstrate your changes work as intended.

2. **Documentation**
   - Update documentation if necessary.  
   - Currently, documentation lives in [`openapi/openapi.yaml`](./openapi/openapi.yaml).

3. **Code Style & Formatting**
   - Rust code:  

     ```bash
     cargo fmt
     ```

   - TypeScript code (in `tests/`):  

     ```bash
     npm run format
     ```

4. **Linting**
   - Rust (using the latest `clippy`):  

     ```bash
     cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings
     ```

   - TypeScript (in `tests/`):  

     ```bash
     npm run lint
     ```

5. **Commit Messages**
   - Follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.

## Checks Before PRs

Pull requests are automatically tested using GitHub Actions.  
For a pull request to be merged, the following checks must pass:

- Build must succeed
- All tests must pass
- All lint checks must pass

It is recommended to run these checks locally before opening a PR.

## Questions?

If anything is unclear, feel free to open an issue or ask in the pull request discussion.  
