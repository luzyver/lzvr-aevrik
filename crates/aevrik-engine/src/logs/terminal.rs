// Filter terminal commands before framing lines, including commands split across reads.
#[derive(Default)]
pub(super) struct Terminal(State);

#[derive(Default)]
enum State {
    #[default]
    Text,
    Escape,
    Csi,
    String,
    StringEscape,
    Intermediate,
}

impl Terminal {
    pub fn text_byte(&mut self, byte: u8) -> bool {
        // A malformed command must not swallow subsequent log lines.
        if byte == b'\n' {
            self.0 = State::Text;
            return true;
        }
        self.0 = match self.0 {
            State::Text => match byte {
                0x1b => State::Escape,
                b'\t' | b'\r' | 0x20..=0x7e | 0x80..=0xff => return true,
                _ => State::Text,
            },
            State::Escape => match byte {
                b'[' => State::Csi,
                b']' | b'P' | b'X' | b'^' | b'_' => State::String,
                0x20..=0x2f => State::Intermediate,
                _ => State::Text,
            },
            State::Csi => match byte {
                0x40..=0x7e => State::Text,
                0x1b => State::Escape,
                _ => State::Csi,
            },
            State::String => match byte {
                0x07 => State::Text,
                0x1b => State::StringEscape,
                _ => State::String,
            },
            State::StringEscape => match byte {
                b'\\' | 0x07 => State::Text,
                0x1b => State::StringEscape,
                _ => State::String,
            },
            State::Intermediate => match byte {
                0x30..=0x7e => State::Text,
                _ => State::Intermediate,
            },
        };
        false
    }
}

pub(super) fn message(text: &str) -> &str {
    let bytes = text.as_bytes();
    // Astro/Vite prefix their own timestamp. Keep dates and timestamps in message bodies.
    if bytes.len() > 9
        && [0, 1, 3, 4, 6, 7]
            .iter()
            .all(|&i| bytes[i].is_ascii_digit())
        && matches!(bytes[2], b'.' | b':')
        && bytes[5] == bytes[2]
        && bytes[8] == b' '
        && text[9..].starts_with('[')
    {
        &text[9..]
    } else {
        text
    }
}

pub(super) fn level<'a>(text: &str, fallback: &'a str) -> &'a str {
    let text = text.trim_start();
    if text.starts_with("[ERROR]") || text.starts_with("[FATAL]") {
        "ERROR"
    } else if text.starts_with("[WARN]")
        || text.starts_with("[WARNING]")
        || text.starts_with("Warning:")
        || text.starts_with("DeprecationWarning:")
        || (text.starts_with("(node:") && text.contains("Warning:"))
        || (text.starts_with("(Use `node --trace-")
            && text.ends_with("to show where the warning was created)"))
    {
        "WARN"
    } else {
        fallback
    }
}
