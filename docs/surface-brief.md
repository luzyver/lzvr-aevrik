# Aevrik — Windows environment workspace

Mode: Operate. Build path: code-led interactive HTML prototype. Platform implementation: web; intended product platform: Windows desktop.

## User direction
“btw ini aplikasi windows kan? kok desainnya kayak web app ya”
“oke revisi, trus cari warna yang bagus deh, gue kurang suka hijau, trus cari nama yang keren dan tidak mainstream, serta pakai inggris/bahasa asing juga oke”

## Direction contract
THESIS: A Windows utility for inspecting and controlling the local stack. A service list and its properties share the main workspace; service actions appear in the output log.
OWN-WORLD: Windows utility conventions: integrated window title/caption controls, compact navigation, neutral layered surfaces, Segoe UI, command buttons, aligned tables, persistent property panes and bottom status. Graphite #1c1c20 / #242428 with iris #b8adff; light pearl #f8f8fa with #6550b5. No green. These are proposed brand colors, not mandated Microsoft colors.
STORY: Select a service, read port/version/status, start or stop it, see the log update. Switch to Projects to manage project-specific runtime and HTTPS without a blocking drawer.
FIRST VIEWPORT: App window against a neutral desktop stage; title bar above navigation and workspace; service table left, properties right, Output below and status at the window bottom. At 1440x900 the first output lines are visible. Smaller windows scroll the content pane while preserving navigation and status.
FORM: seed:abd7183e7044a276 is retained as provenance of the prior mockup. Its forest workshop interpretation is superseded by explicit user-pinned Windows desktop direction; no new random concept roll is used to override the user. The prior seed was corroborated during the previous review. The new visual authority is user direction plus the Microsoft sources in REFERENCES.md.
SIGNATURE: Persistent master/detail service inspector with synchronized status, command buttons and output, plus a reversible window-size preview. Service controls, titlebar actions and preferences are explicitly simulated.
CONTENT: Four fictional projects, six service entries, nine package entries and sample logs. No runtime-version freshness claims.
BEHAVIOR: Seven routes; runtime changes; filters/search/empty states; simulated start/stop/restart; project creation and validation; installation loading state; HTTPS switch; Ctrl+K/Enter/Escape; log export/clear; theme; window preview controls.
RESPONSIVE: 192px navigation at full desktop, icon navigation below 960px, stacked list/properties below 700px. Table service rows reorganize to keep status, port and controls visible. Domains retain a labelled horizontal-scroll region. Compact mode is a resizable-window fallback, not a separate mobile product.
QUALITY: Functional controls use at least 12px text. Labels accompany status colors. Focus rings, neutral SVG icons, no decorative effects. Real browser testing at 1440/1280/1024/768/390. All sample state changes remain in browser memory.
FINISH: Independent finish review followed by design-system documentation and a portable ZIP.

## Reference authority
Microsoft title bar, navigation and color guidance, linked in REFERENCES.md. Herd remains a workflow reference. No approved raster comp or quality-bar image exists for this code-led redesign.
