Aevrik - Windows x64 portable application

Requires Microsoft Edge WebView2 Runtime. Rust and Node.js are not needed to run Aevrik.
Close any older Aevrik window before opening the new executable.

Runtimes > choose a version > Install lets you type an installation folder or Browse.
For example: D:\Aevrik\runtimes. Downloads and extracted runtime files use that drive.
The chosen folder is remembered. Existing versions stay in their current locations.
Configure service fills the selected executable path; supply its foreground launch arguments.

Uninstall permanently deletes only the selected managed version folder, including any data
you placed inside it. Keep database data and personal configuration outside runtime folders.
Referenced runtimes cannot be uninstalled: stop the service and Remove registration first.
Removing a registration unlinks projects but preserves files and installed runtimes.

Application identifier: id.luzyver.aevrik.desktop.
On first launch, choose a data folder or accept %APPDATA%\id.luzyver.aevrik.desktop.
Settings > Data folder > Change folder saves a location for the next launch.
Switching folders does not move existing files; select the old folder to reopen its workspace.
A small data-location.json pointer stays in the Windows application-config folder.
Upgrades keep %APPDATA%\dev.aevrik.desktop when it contains the old database, so saved paths remain valid.
AEVRIK_DATA_DIR overrides the saved choice and disables changing it in Settings. Metadata is separate from runtime storage.

Database setup, graceful database shutdown, automatic per-project switching, DNS/HTTPS and
automatic updates remain unfinished. Stop/Restart/app exit force-terminate owned processes.
Shut down database servers through their own tools first. This EXE is unsigned.

This EXE is portable and does not install itself. For Setup with folder selection and
a Windows uninstall entry, download the separate -setup.exe release asset.
