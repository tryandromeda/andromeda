// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use reedline::{ValidationResult, Validator};

/// JavaScript syntax validator for multiline input
#[derive(Clone)]
pub struct JsValidator;

impl Validator for JsValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let trimmed = line.trim();

        // Special commands are always complete
        if matches!(
            trimmed,
            "help" | "exit" | "quit" | "clear" | "history" | "gc" | ""
        ) {
            return ValidationResult::Complete;
        }

        if is_incomplete_js(line) {
            ValidationResult::Incomplete
        } else {
            ValidationResult::Complete
        }
    }
}

fn is_incomplete_js(code: &str) -> bool {
    let mut paren_count = 0;
    let mut brace_count = 0;
    let mut bracket_count = 0;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut escape_next = false;
    let mut in_single_comment = false;
    let mut in_multi_comment = false;

    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if escape_next {
            escape_next = false;
            i += 1;
            continue;
        }

        if in_single_comment {
            if ch == '\n' {
                in_single_comment = false;
            }
            i += 1;
            continue;
        }

        if in_multi_comment {
            if ch == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                in_multi_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if in_string {
            if ch == '\\' {
                escape_next = true;
            } else if ch == string_char {
                in_string = false;
                string_char = '\0';
            }
            i += 1;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = ch;
            }
            '/' if i + 1 < chars.len() => {
                if chars[i + 1] == '/' {
                    in_single_comment = true;
                    i += 1;
                } else if chars[i + 1] == '*' {
                    in_multi_comment = true;
                    i += 1;
                }
            }
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            _ => {}
        }

        i += 1;
    }

    paren_count > 0 || brace_count > 0 || bracket_count > 0 || in_string || in_multi_comment
}
