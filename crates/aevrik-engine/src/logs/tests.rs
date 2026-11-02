use super::*;
use std::io::Cursor;

struct Fragmented(Cursor<Vec<u8>>);
impl Read for Fragmented {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(&mut buffer[..1])
    }
}

#[test]
fn handles_fragmented_commands_unicode_and_unterminated_lines() {
    let logs = Logs::default();
    let input = format!(
        "\x1b]0;hidden title\x07\x1b[32m{}┃\x1b[0m done\n\x1b[123\n[ERROR] next line\nlast",
        "x".repeat(8191)
    );
    logs.read_output(Fragmented(Cursor::new(input.into_bytes())), "test", "INFO");
    let entries = logs.snapshot();
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].message, format!("{}┃", "x".repeat(8191)));
    assert_eq!(entries[1].message, " done");
    assert!(
        entries
            .iter()
            .all(|entry| !entry.message.contains(['\x1b', '\u{fffd}']))
    );
    assert_eq!(entries[entries.len() - 2].level, "ERROR");
    assert_eq!(entries.last().unwrap().message, "last");
}

#[test]
fn preserves_message_timestamps_and_unknown_stderr() {
    let logs = Logs::default();
    logs.read_output(
        Cursor::new("16:41:22 appointment\n2026-11-02T16:41:22 [INFO] event\n  details\n"),
        "test",
        "ERROR",
    );
    let entries = logs.snapshot();
    assert_eq!(entries[0].message, "16:41:22 appointment");
    assert_eq!(entries[1].message, "2026-11-02T16:41:22 [INFO] event");
    assert_eq!(entries[2].message, "  details");
    assert!(entries.iter().all(|entry| entry.level == "ERROR"));
}

#[test]
fn cleans_real_astro_output_without_losing_diagnostics() {
    let logs = Logs::default();
    logs.read_output(
        Cursor::new(concat!(
            "\x1b[2m16.41.22\x1b[22m \x1b[34m[vite]\x1b[39m connected.\r\n",
            "\x1b[0m\r\n   \n",
            "16.41.35 [WARN] [astro-icon] Failed to load icons\n",
            "16.41.42 [404] /api/github-contributions\n",
            "  at handler (D:\\project\\main.js:12)\n",
            "┃ Local \x1b]8;;http://localhost:4321/\x1b\\http://localhost:4321/\x1b]8;;\x07"
        )),
        "portfolio",
        "INFO",
    );
    let entries = logs.snapshot();
    assert_eq!(entries.len(), 5);
    assert_eq!(entries[0].message, "[vite] connected.");
    assert_eq!(entries[1].level, "WARN");
    assert_eq!(entries[2].message, "[404] /api/github-contributions");
    assert_eq!(entries[3].message, "  at handler (D:\\project\\main.js:12)");
    assert_eq!(entries[4].message, "┃ Local http://localhost:4321/");
}

#[test]
fn distinguishes_explicit_warnings_from_stderr_errors() {
    let logs = Logs::default();
    logs.read_output(
        Cursor::new(concat!(
            "(node:17680) [DEP0205] DeprecationWarning: module.register() is deprecated.\n",
            "(Use `node --trace-deprecation ...` to show where the warning was created)\n",
            "Error: failed to load warning configuration\n",
            "[ERROR] warning handler failed\n"
        )),
        "portfolio",
        "ERROR",
    );
    let entries = logs.snapshot();
    assert_eq!(
        entries.iter().map(|e| e.level.as_str()).collect::<Vec<_>>(),
        ["WARN", "WARN", "ERROR", "ERROR"]
    );
}
