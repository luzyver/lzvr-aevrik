# Native runtime catalog and fixed workspace

Mode: Operate. Target: `desktop/` within the Windows Tauri executable. The earlier standalone browser mockup and its simulation scripts have been removed.

User request: stop scrolling the entire workspace into blank space, expose installable runtimes, and add a broad choice of products and versions.

Preserve graphite/iris and pearl tokens, Segoe UI, integrated caption controls, compact navigation, persistent service properties and bottom status. Seed provenance `seed:abd7183e7044a276` belongs to the original mockup; no new visual-world roll is used for this functional correction.

The native main area fills its available height and never scrolls. Service/project lists, inspectors, full output, settings and runtime catalog own their bounded scroll areas. Minimum supported window: 800×600. Check 1280×820 and 1440×1000 as well.

The catalog contains 14 product rows and 140 pinned releases. Each row shows a labelled version selector, version/installed counts, category, publisher, Windows architecture, description and Install or Configure service plus Uninstall for the selected release. nginx is x86; the other builds are x64. Versions install separately; switching selection never changes an existing service. Search includes versions and categories. Download status includes byte counts and a progress bar where a total is available. Verification/extraction is indeterminate. Failures retain an actionable error and Retry. Only the Rust completion marker produces Installed and Configure service.

Install opens a labelled folder input with native Browse, prefilled from the last choice, including another drive. It explains the per-release subfolder and that existing versions stay in place. Uninstall names the exact folder, explicitly confirms deletion of everything inside it and reports blocked references inline. Remove registration is available only for stopped services and has its own confirmation; it preserves files. Dialogs remain readable at the 800×600 minimum window.

First-run Services offers Browse packages while keeping manual registration in the command bar. Configure service fills the installed executable and parent directory; the person chooses launch arguments and working directory. Installation never starts a runtime or initializes a database.

Evidence lives in `previews/desktop/`: catalog screenshots, package download verification, and real packaged-window regression results. Current presentation tokens are maintained directly in `styles/` and `desktop/native.css`.
