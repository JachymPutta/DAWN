# What is this?

DAWN (Debug Adapter with Nix) is an implementation of the DAP adapter for the nix debugger.

# Configuration

Configuration for Neovim:

```lua
-- configure the dap adapter
local dap = require("dap")
dap.adapters.nix = {
    type = "executable",
    executable = {
        command = "", --TODO: add command
        args = {},
    },
}
-- configure the dap configuration
dap.configurations.nix = {
    type = "nix",
    request = "launch",
    name = "Launch Program (nix debug adapter)",
    program = "$${file}",
}
```
# Usage
No usage yet, still WIP.

# Design

- DAWN crate
    - Build overarching .so file that builds as plugin for rust
- REP
    - Figure out frontend rep. I think it will match the DAP rep, so keeping it in the same crate makes sense

- Figure out how to communicate (e.g. which port, etc). It might be possible to do over stdin/stdout, but likely would need to check.
- Figure out how to get stuff out of nix.

- Add debugger flag
- initialize a NixRepl in a sensible enough way

# Thanks
Thanks to the Hiro Systems team, who maintains the debug adapter infrastructure that is heavily used by the dap-server crate.
