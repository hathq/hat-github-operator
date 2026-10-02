# hat-github-operator

Turn an explicitly authorized role invocation into a supported GitHub operation.

## What you can do

- Validate exact package, binding, target and permission.
- Request private repository creation, snapshot push or declared issue/PR operations.

## Current scope

Deletion requires an exact grant and backup declaration. Public visibility changes, force pushes and permission-setting changes are outside the implemented operation set.

Package distribution is not activated by this documentation. Use the checked-in source and the declared dependency versions; published availability must be verified separately.

## Getting started

Install Rust 1.97 or newer and make the declared dependencies available. Use the configured private registry when a dependency is not distributed publicly. Run from this repository:

```sh
cargo test --locked
```

## Documentation and source

[Usage guide](docs/getting-started.md)

[Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)
