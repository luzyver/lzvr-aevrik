# Aevrik 0.1.0-beta-1

- Choose and remember a runtime installation folder, including another drive such as `D:\Aevrik\runtimes`. Downloads and extraction stay on that drive.
- Uninstall a specific managed version with explicit confirmation. Referenced runtimes and junction/symlink trees are protected. Remove stopped service registrations without deleting their files.
- Expand to 19 products and 179 pinned releases, including .NET SDK, .NET Runtime, ASP.NET Core Runtime, Zig and Nim. Nim requires a separate C compiler.
- Check for runtime catalog updates and install newer versions into a chosen folder without replacing existing versions or switching services.
- Check Aevrik releases in Settings and open the download page for manual installation.
- Stream downloads to disk with SHA-256/SHA-512 validation, a 2 GiB download limit and an 8 GiB extraction limit.
- Adopt `id.luzyver.aevrik.desktop`; retain existing legacy metadata and runtime paths in place.
- Build EXE/ZIP, run native smoke tests and publish tagged releases on GitHub-hosted Windows runners.

Portable unsigned Windows x64 executable; WebView2 is required. Database initialization, graceful shutdown, automatic per-project switching, DNS/HTTPS and automatic app updates remain unavailable. Stop/Restart/exit terminate the owned process tree. Keep database data outside runtime installation folders.
