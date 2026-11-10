# Product

<!-- impeccable:product-schema 1 -->

> Draft written from the repository (README, source, screenshots) without an interview. Every statement below is inferred from the code and still needs the owner's confirmation.

## Platform

web

## Users

(Inferred) Web developers on Windows who run several local projects side by side: a Node.js app with an npm script, a PHP site, a database, a cache. They open Aevrik at the start of a working session, start what they need, and leave it in the tray while they work in an editor and a terminal. They come back when something will not start, a port is taken, or a runtime needs installing or updating.

## Product Purpose

(Inferred) Aevrik is a local development environment manager for Windows. It keeps a list of projects, each with a folder, a `.test` domain label and a runtime, and the services that belong to them. It starts, stops and watches those services, shows their output, and installs and updates the runtimes they need. Success: a developer gets every service of a project running with one action, and sees at a glance which ones are running, stopped or have exited with an error.

## Positioning

(Inferred) One native Windows app that owns the processes it starts: services keep running when the window is closed to the tray, a service that exits by itself is reported with its exit code, and runtimes are installed from verified archives into a data folder the user chooses, without touching the system PATH.

## Operating Context

(Inferred) Windows 10/11 desktop, a Tauri window that ships at 1280×820 and must work at 800×600. Used alongside an editor, a terminal and a browser, often on a second monitor or half a screen. Tray icon with Open, Start all, Stop all and Exit. Keyboard users reach everything through Ctrl+K.

## Capabilities and Constraints

- Projects: add, edit, remove; folder, name, `.test` domain label, runtime label, linked service.
- Services: add, edit, remove; command, arguments, working folder, expected TCP port; Start/Stop per row, Start all/Stop all; states Running, Stopped, Exited · code N.
- npm projects: set up a service from a package.json script.
- Runtimes: catalog by category, install, set default, update, remove.
- Output: per-service log with follow, filter and copy.
- Settings: theme (dark, light, system), data folder, Start with Windows, Restore services, app updates.
- Open in browser, open folder, open terminal from a project.
- Not implemented: cancel/resume downloads, pnpm/Yarn, DNS/HTTPS for `.test` domains, database initialisation, per-service tray items.
- Technical: Rust engine, Tauri 2 shell, vanilla TypeScript frontend that renders HTML strings; no UI framework.

## Brand Commitments

- Name: Aevrik. Mark: three trapezoids forming an A (`assets/mark.svg`).
- Interface language: English, plain utility wording ("Start", "Stop all", "Exit and stop services").
- Undecided: whether the current graphite/pearl look with an iris accent stays. A visual redesign is being explored.

## Evidence on Hand

- Current interface screenshots in `previews/desktop/`.
- No customers, benchmarks or testimonials exist; none may be invented.

## Product Principles

- State before decoration: what is running, what stopped and what failed is readable at a glance.
- One action to get to work: starting a project's services never takes a detour.
- Nothing destructive without a clear confirmation (exit, remove).
- Fully usable from the keyboard.
