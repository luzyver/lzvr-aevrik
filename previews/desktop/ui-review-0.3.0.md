# Aevrik 0.3.0 catalog review

Scope: extension of the accepted Windows utility UI. Inspected the real release WebView2 captures at 1280x820 and the compact 800x600 outer window, with graphite and pearl themes. No redesign or new visual world was introduced.

The product row groups versions into a labelled native selector. Version counts, Installed state, progress, errors/retry and Configure service stay tied to release IDs. Source/architecture text remains legible, long installed paths wrap, and compact controls remain within the content pane. The runtime list alone scrolls; titlebar/statusbar stay fixed. Empty/error/search and selection-during-download behavior are covered by the mocked UI suite; real installation/restart and native resizing are covered separately.

Existing Segoe UI, neutral layered surfaces, iris controls and compact navigation are preserved. Shared tokens, DESIGN.md and its sidecar were not changed. Their pre-existing timestamp drift was left alone. Seed provenance remains seed:abd7183e7044a276, corroborated by .impeccable/questions/roll.json. No new detector ignores were added.

Evidence: native-catalog-available-0.3.0.png, native-catalog-installed-0.3.0.png, native-catalog-800-0.3.0.png and native-package-verification-0.3.0.json. Source review was performed inline, not by an independent reviewer.
