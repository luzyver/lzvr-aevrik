# Terminal runtime defaults

In **Runtimes**, select an installed version, then **Set as default**. Supported
families: Node.js, Python, PHP, Bun, Deno, Zig and Nim. Go, Java and the .NET
distributions need separate toolchain/environment resolution and are not yet
included in this action.

The confirmation shows the executable, the current PATH command, and the scope:
the current Windows account when no system command takes priority, otherwise
all Windows users. Updating the system PATH requires running Aevrik as
Administrator. Aevrik does not elevate itself or silently change the scope.

After applying, restart the terminal and any editor hosting terminals. If a
parent process still retains the old environment, restart that process too
(or sign out and in). The **Terminal default** label reflects the persisted
Windows PATH, not the environment already inherited by an open terminal.
Shell aliases, the current working directory and active virtual environments
can still override command resolution. Python's `py` launcher has separate
version selection; use `python -m pip` with the selected interpreter.

Python adds its executable folder and its `Scripts` folder. Packages in an
existing virtual environment stay there; changing the default does not copy
them or replace the interpreter of that environment. Explicit service and
project executable paths are unchanged.

The selected folders are prepended to one PATH scope. Unrelated entries retain
their exact text and order, including expandable variables; older runtimes
remain as fallbacks. **Remove from PATH** removes only this version's folders
from the scope shown in the confirmation. If both scopes contain the folders,
repeat for the other scope. A previous runtime may become the default again.
This does not uninstall the runtime. Uninstall refuses any runtime folder
still referenced by a persistent PATH entry.

Before writing, the engine recomputes the plan from the installed package and
the live registry. A stale confirmation is rejected. Writes retain the registry
value type, save the previous PATH to `path-backup-<id>.json` under the runtime
metadata root, and broadcast the Windows environment-change notification.
Backups contain the previous scope, raw PATH, registry kind and whether the
value existed. No other environment values are saved. These are recovery
records, not an automatic full-PATH rollback: restoring one after later edits
requires reconciling those edits.

Verification: Node tests cover IPC intent, browser tests cover confirmation,
permission failure, busy state, default labels and removal; Rust tests cover
PATH edits and scope planning. Rust compilation/tests run in the existing
Windows workflow. Browser tests mock IPC and never edit the host PATH.
