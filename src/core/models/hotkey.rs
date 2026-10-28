#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalHotkeyAction {
    Capture,
    ToggleRecording,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotkeyBinding {
    pub action: GlobalHotkeyAction,
    pub accelerator: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HotkeyRegistration {
    pub hotkey_id: u32,
    pub action: GlobalHotkeyAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyRegistrationFailure {
    pub action: GlobalHotkeyAction,
    pub accelerator: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySymbolStyle {
    MacSymbols,
    Words,
}

const ACCELERATOR_SEPARATOR: char = '+';
const WORD_STYLE_JOINER: &str = " + ";
const MAC_STYLE_JOINER: &str = "";
const KEY_CODE_PREFIX: &str = "KEY";
const DIGIT_CODE_PREFIX: &str = "DIGIT";

struct ModifierDisplayEntry {
    accepted_names: &'static [&'static str],
    mac_symbol: &'static str,
    word: &'static str,
}

const MODIFIER_DISPLAY_TABLE: &[ModifierDisplayEntry] = &[
    ModifierDisplayEntry {
        accepted_names: &["ALT", "OPTION"],
        mac_symbol: "⌥",
        word: "Alt",
    },
    ModifierDisplayEntry {
        accepted_names: &["SHIFT"],
        mac_symbol: "⇧",
        word: "Shift",
    },
    ModifierDisplayEntry {
        accepted_names: &["CTRL", "CONTROL"],
        mac_symbol: "⌃",
        word: "Ctrl",
    },
    ModifierDisplayEntry {
        accepted_names: &["CMD", "COMMAND", "SUPER", "META"],
        mac_symbol: "⌘",
        word: "Win",
    },
];

pub fn current_platform_key_symbol_style() -> KeySymbolStyle {
    match cfg!(target_os = "macos") {
        true => KeySymbolStyle::MacSymbols,
        false => KeySymbolStyle::Words,
    }
}

pub fn format_accelerator_for_display(accelerator: &str, style: KeySymbolStyle) -> String {
    let display_tokens: Vec<String> = accelerator
        .split(ACCELERATOR_SEPARATOR)
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| format_accelerator_token(token, style))
        .collect();
    let joiner = match style {
        KeySymbolStyle::MacSymbols => MAC_STYLE_JOINER,
        KeySymbolStyle::Words => WORD_STYLE_JOINER,
    };
    display_tokens.join(joiner)
}

fn format_accelerator_token(token: &str, style: KeySymbolStyle) -> String {
    let uppercase_token = token.to_uppercase();
    MODIFIER_DISPLAY_TABLE
        .iter()
        .find(|entry| entry.accepted_names.contains(&uppercase_token.as_str()))
        .map(|entry| match style {
            KeySymbolStyle::MacSymbols => entry.mac_symbol.to_string(),
            KeySymbolStyle::Words => entry.word.to_string(),
        })
        .unwrap_or_else(|| strip_key_prefix(&uppercase_token))
}

fn strip_key_prefix(uppercase_token: &str) -> String {
    let without_key_prefix = uppercase_token
        .strip_prefix(KEY_CODE_PREFIX)
        .filter(|remainder| remainder.len() == 1)
        .unwrap_or(uppercase_token);
    let without_digit_prefix = without_key_prefix
        .strip_prefix(DIGIT_CODE_PREFIX)
        .filter(|remainder| remainder.len() == 1)
        .unwrap_or(without_key_prefix);
    capitalize_named_key(without_digit_prefix)
}

fn capitalize_named_key(uppercase_token: &str) -> String {
    let mut characters = uppercase_token.chars();
    match characters.next() {
        Some(first_character) => {
            format!("{}{}", first_character, characters.as_str().to_lowercase())
        }
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_accelerator_mac_symbols() {
        let formatted = format_accelerator_for_display("Alt+Shift+S", KeySymbolStyle::MacSymbols);
        assert_eq!(formatted, "⌥⇧S");
    }

    #[test]
    fn test_format_accelerator_words() {
        let formatted = format_accelerator_for_display("ctrl+shift+KeyR", KeySymbolStyle::Words);
        assert_eq!(formatted, "Ctrl + Shift + R");
    }

    #[test]
    fn test_format_accelerator_ignores_empty_tokens() {
        let formatted = format_accelerator_for_display("Alt++S", KeySymbolStyle::Words);
        assert_eq!(formatted, "Alt + S");
    }

    #[test]
    fn test_format_accelerator_keeps_named_keys() {
        let formatted = format_accelerator_for_display("Cmd+Digit5", KeySymbolStyle::MacSymbols);
        assert_eq!(formatted, "⌘5");
    }

    #[test]
    fn test_named_keys_are_capitalized_once() {
        assert_eq!(
            format_accelerator_for_display("Escape", KeySymbolStyle::Words),
            "Escape"
        );
        assert_eq!(
            format_accelerator_for_display("esc", KeySymbolStyle::Words),
            "Esc"
        );
    }

    #[test]
    fn test_format_accelerator_empty_input() {
        assert_eq!(
            format_accelerator_for_display("", KeySymbolStyle::Words),
            ""
        );
    }
}
