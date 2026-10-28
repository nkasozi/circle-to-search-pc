const MODIFIER_KEY_CODE_PREFIXES: [&str; 5] = ["Shift", "Control", "Alt", "Super", "Meta"];
const FUNCTION_KEY_PREFIX: char = 'F';
const KEY_CODE_PREFIX: &str = "Key";
const DIGIT_CODE_PREFIX: &str = "Digit";
const ALLOWED_NAMED_CODES: [&str; 5] = ["Space", "Backquote", "Minus", "Equal", "Slash"];
const CONTROL_TOKEN: &str = "Ctrl";
const ALT_TOKEN: &str = "Alt";
const SHIFT_TOKEN: &str = "Shift";
const COMMAND_TOKEN: &str = "Cmd";
const ACCELERATOR_SEPARATOR: &str = "+";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PressedModifiers {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub command: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutCaptureOutcome {
    StillWaiting,
    NeedsModifier,
    Unsupported,
}

fn is_function_key(code: &str) -> bool {
    code.strip_prefix(FUNCTION_KEY_PREFIX)
        .is_some_and(|number| {
            !number.is_empty() && number.chars().all(|character| character.is_ascii_digit())
        })
}

fn is_supported_main_key(code: &str) -> bool {
    let is_letter = code
        .strip_prefix(KEY_CODE_PREFIX)
        .is_some_and(|letter| letter.len() == 1);
    let is_digit = code
        .strip_prefix(DIGIT_CODE_PREFIX)
        .is_some_and(|digit| digit.len() == 1);
    is_letter || is_digit || is_function_key(code) || ALLOWED_NAMED_CODES.contains(&code)
}

pub fn build_accelerator(
    modifiers: PressedModifiers,
    physical_code: &str,
) -> Result<String, ShortcutCaptureOutcome> {
    if MODIFIER_KEY_CODE_PREFIXES
        .iter()
        .any(|prefix| physical_code.starts_with(prefix))
    {
        return Err(ShortcutCaptureOutcome::StillWaiting);
    }
    if !is_supported_main_key(physical_code) {
        return Err(ShortcutCaptureOutcome::Unsupported);
    }
    let modifier_tokens: Vec<&str> = [
        (modifiers.control, CONTROL_TOKEN),
        (modifiers.alt, ALT_TOKEN),
        (modifiers.shift, SHIFT_TOKEN),
        (modifiers.command, COMMAND_TOKEN),
    ]
    .into_iter()
    .filter_map(|(is_pressed, token)| is_pressed.then_some(token))
    .collect();
    let has_meaningful_modifier = modifiers.control || modifiers.alt || modifiers.command;
    if !has_meaningful_modifier && !is_function_key(physical_code) {
        return Err(ShortcutCaptureOutcome::NeedsModifier);
    }
    Ok(modifier_tokens
        .into_iter()
        .chain(std::iter::once(physical_code))
        .collect::<Vec<&str>>()
        .join(ACCELERATOR_SEPARATOR))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alt_shift() -> PressedModifiers {
        PressedModifiers {
            alt: true,
            shift: true,
            ..PressedModifiers::default()
        }
    }

    #[test]
    fn test_letter_with_modifiers_builds_accelerator() {
        assert_eq!(
            build_accelerator(alt_shift(), "KeyS"),
            Ok("Alt+Shift+KeyS".to_string())
        );
    }

    #[test]
    fn test_modifier_only_press_keeps_waiting() {
        assert_eq!(
            build_accelerator(alt_shift(), "ShiftLeft"),
            Err(ShortcutCaptureOutcome::StillWaiting)
        );
        assert_eq!(
            build_accelerator(alt_shift(), "AltRight"),
            Err(ShortcutCaptureOutcome::StillWaiting)
        );
    }

    #[test]
    fn test_plain_letters_and_shift_only_need_a_modifier() {
        assert_eq!(
            build_accelerator(PressedModifiers::default(), "KeyA"),
            Err(ShortcutCaptureOutcome::NeedsModifier)
        );
        let shift_only = PressedModifiers {
            shift: true,
            ..PressedModifiers::default()
        };
        assert_eq!(
            build_accelerator(shift_only, "KeyA"),
            Err(ShortcutCaptureOutcome::NeedsModifier)
        );
    }

    #[test]
    fn test_function_keys_work_alone() {
        assert_eq!(
            build_accelerator(PressedModifiers::default(), "F5"),
            Ok("F5".to_string())
        );
    }

    #[test]
    fn test_unsupported_keys_are_rejected() {
        assert_eq!(
            build_accelerator(alt_shift(), "ArrowUp"),
            Err(ShortcutCaptureOutcome::Unsupported)
        );
        assert_eq!(
            build_accelerator(alt_shift(), "Fn"),
            Err(ShortcutCaptureOutcome::Unsupported)
        );
    }

    #[test]
    fn test_command_modifier_order() {
        let modifiers = PressedModifiers {
            control: true,
            command: true,
            shift: true,
            alt: false,
        };
        assert_eq!(
            build_accelerator(modifiers, "Digit2"),
            Ok("Ctrl+Shift+Cmd+Digit2".to_string())
        );
    }
}
