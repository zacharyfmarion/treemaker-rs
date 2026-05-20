# oristudio-cp

`oristudio-cp` is the planned Oriedita-compatible crease-pattern editing
kernel for Ori Studio.

The crate starts with a deliberately conservative contract: every known
Oriedita non-UI operation is registered, but operations return typed
unsupported errors until the behavior is directly ported and validated against
the pinned Oriedita oracle.

The implementation roadmap lives in:

- `implementation-plans/oriedita-port.md`
- `implementation-plans/oriedita-source-map.md`
