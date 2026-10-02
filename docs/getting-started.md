# Using hat-github-operator

Turn an explicitly authorized role invocation into a supported GitHub operation.

## Before you start

Deletion requires an exact grant and backup declaration. Public visibility changes, force pushes and permission-setting changes are outside the implemented operation set.

## First steps

Run from the repository root:

```sh
cargo test --locked
```

## How to assess the result

- Validate exact package, binding, target and permission.
- Request private repository creation, snapshot push or declared issue/PR operations.

A passing source-level check establishes only what that check observes. Keep missing configuration, unavailable services and unverified deployment paths visible.

## Continue reading

[Repository overview](../README.md)
