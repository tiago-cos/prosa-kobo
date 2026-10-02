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
   - Ensure all tests pass. Instructions are in
     [Contributing and Architecture](https://github.com/tiago-cos/prosa-kobo/wiki/Contributing-and-Architecture).
   - Add tests that demonstrate your changes work as intended.

2. **Documentation**
   - Update documentation if necessary. It lives in two places:
     - The API reference is the OpenAPI spec in [`openapi/`](../openapi). Keep it
       in step with any endpoint you add or change.
     - Everything else — installing, configuring, connecting a Kobo and working
       on Prosa-Kobo — is in [`wiki/`](../wiki), which is published to the
       [wiki](https://github.com/tiago-cos/prosa-kobo/wiki) by a workflow. Edit
       the files in `wiki/`, not the wiki itself.

3. **Code Style & Formatting**
   - Rust code:  

     ```bash
     cargo fmt
     ```

4. **Linting**
   - Rust (using the latest `clippy`):  

     ```bash
     cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings
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
