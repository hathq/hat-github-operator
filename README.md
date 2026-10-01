# GitHub Operator HAT

An explicitly installed GitHub integration HAT. Its declarations distinguish private repository creation, immutable source snapshot push, Issue and Pull Request creation, workflow dispatch, and private repository deletion permissions.

The package validates HAT identity, binding, runtime grant, permission, and expiry, then converts an authorized invocation into the declared `zixcel://github/api-request/v1` contract. The declared Zixcel dependency performs the API request.

Grant signatures are verified through the `HatGrantVerifier` port. Connection secret resolution and transport remain with their configured providers; credentials are not package content. Deletion requires an exact grant with the expected repository ID and backup declaration. Public visibility changes, force push, and GitHub permission-setting changes are outside the implemented operation set.

## Responsibilities

- Bound operations by the installed HAT package permission set.
- Match package, binding, target, time, and permission references exactly.
- Convert validated invocations to the provider API request contract.
- Receive provider results through the declared interface.
