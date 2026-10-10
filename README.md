# hat-github-operator

Turn an explicitly authorized role invocation into a supported GitHub operation.

## What you can do

- Validate exact package, binding, target and permission.
- Request private repository creation, snapshot push or declared issue/PR operations.

## Current scope

Deletion requires an exact grant and backup declaration. Public visibility changes, force pushes and permission-setting changes are outside the implemented operation set.

Package distribution is not activated by this documentation. Use the checked-in source and the declared dependency versions; published availability must be verified separately.

## Getting started

Install Rust 1.97 or newer and make the declared dependencies available. Resolve the declared library dependencies from crates.io. Run from this repository:

```sh
cargo test --locked
```

## Documentation and source

[Usage guide](docs/getting-started.md)

[Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)

## Provider composition

The worker uses `--zixcel-github-transport` and `--zixcel-transport-config` to
select the GitHub provider executable and its owner-private configuration.
GitHub egress contracts resolve from crates.io. HAT specifications also resolve from crates.io. Caller
authorization must name the exact `zixcel-github-transport` audience; no
credential, implicit permission, or provider mutation is supplied by this worker.
