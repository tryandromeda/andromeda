// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use reedline::{Highlighter, StyledText};

/// JavaScript syntax highlighter for the REPL
#[derive(Clone)]
pub struct JsHighlighter;

impl Highlighter for JsHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut styled = StyledText::new();

        // Handle special REPL commands first
        let trimmed = line.trim();
        if matches!(
            trimmed,
            "help" | "exit" | "quit" | "clear" | "history" | "gc"
        ) {
            styled.push((
                nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Yellow),
                trimmed.to_string(),
            ));
            return styled;
        }

        // Tokenize and highlight JavaScript
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            // Skip whitespace
            if ch.is_whitespace() {
                styled.push((nu_ansi_term::Style::new(), ch.to_string()));
                i += 1;
                continue;
            }

            // Handle strings
            if ch == '"' || ch == '\'' || ch == '`' {
                let string_start = i;
                let quote = ch;
                i += 1;

                // Find the end of the string
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
                    } else if chars[i] == quote {
                        i += 1; // Include closing quote
                        break;
                    } else {
                        i += 1;
                    }
                }

                let string_content: String = chars[string_start..i].iter().collect();
                let style = if quote == '`' {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan) // Template literals
                } else {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Green) // Regular strings
                };
                styled.push((style, string_content));
                continue;
            }

            // Handle comments
            if ch == '/' && i + 1 < chars.len() {
                if chars[i + 1] == '/' {
                    // Single line comment
                    let comment: String = chars[i..].iter().collect();
                    styled.push((
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::DarkGray),
                        comment,
                    ));
                    break;
                } else if chars[i + 1] == '*' {
                    // Multi-line comment
                    let comment_start = i;
                    i += 2;

                    while i + 1 < chars.len() {
                        if chars[i] == '*' && chars[i + 1] == '/' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }

                    let comment: String = chars[comment_start..i].iter().collect();
                    styled.push((
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::DarkGray),
                        comment,
                    ));
                    continue;
                }
            }

            // Handle numbers
            if ch.is_ascii_digit()
                || (ch == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
            {
                let number_start = i;

                // Handle decimal numbers
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }

                let number: String = chars[number_start..i].iter().collect();
                styled.push((
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Blue),
                    number,
                ));
                continue;
            }

            // Handle type annotations (TypeScript)
            if ch == ':' && i + 1 < chars.len() && chars[i + 1].is_whitespace() {
                // This is likely a type annotation, so highlight the colon and continue
                styled.push((
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan),
                    ch.to_string(),
                ));
                i += 1;
                continue;
            }

            // Handle angle brackets for generic types
            if ch == '<' || ch == '>' {
                // Check if this might be a generic type by looking for surrounding context
                let is_likely_generic = if ch == '<' {
                    // Look backwards for function/class/interface names or 'new'
                    let mut j = i.saturating_sub(1);
                    while j > 0 && chars[j].is_whitespace() {
                        j = j.saturating_sub(1);
                    }
                    j > 0 && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '$')
                } else {
                    // Look ahead for typical post-generic syntax
                    i + 1 < chars.len() && (chars[i + 1] == '(' || chars[i + 1].is_whitespace())
                };

                let style = if is_likely_generic {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan)
                } else {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red) // Comparison operators
                };
                styled.push((style, ch.to_string()));
                i += 1;
                continue;
            }

            if ch.is_alphabetic() || ch == '_' || ch == '$' {
                let word_start = i;

                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$')
                {
                    i += 1;
                }

                let word: String = chars[word_start..i].iter().collect();

                let style = match word.as_str() {
                    // JavaScript keywords
                    "const" | "let" | "var" | "function" | "return" | "if" | "else" | "for"
                    | "while" | "do" | "switch" | "case" | "default" | "break" | "continue"
                    | "try" | "catch" | "finally" | "throw" | "new" | "this" | "super"
                    | "class" | "extends" | "import" | "export" | "from" | "as" | "async"
                    | "await" | "yield" | "typeof" | "instanceof" | "in" | "of" | "delete"
                    | "void" => nu_ansi_term::Style::new()
                        .fg(nu_ansi_term::Color::Purple)
                        .bold(),

                    // TypeScript-specific keywords
                    "interface" | "type" | "enum" | "namespace" | "module" | "declare"
                    | "abstract" | "implements" | "private" | "protected" | "public"
                    | "readonly" | "static" | "override" | "keyof" | "infer" | "is" | "asserts"
                    | "satisfies" | "using" | "out" | "accessor" => nu_ansi_term::Style::new()
                        .fg(nu_ansi_term::Color::Purple)
                        .bold(),

                    // Built-in TypeScript types
                    "string" | "number" | "boolean" | "object" | "bigint" | "symbol"
                    | "undefined" | "any" | "unknown" | "never" => nu_ansi_term::Style::new()
                        .fg(nu_ansi_term::Color::Blue)
                        .bold(),

                    // Utility types
                    "Partial"
                    | "Required"
                    | "Readonly"
                    | "Record"
                    | "Pick"
                    | "Omit"
                    | "Exclude"
                    | "Extract"
                    | "NonNullable"
                    | "ReturnType"
                    | "InstanceType"
                    | "Parameters"
                    | "ConstructorParameters"
                    | "ThisParameterType"
                    | "OmitThisParameter"
                    | "ThisType"
                    | "Uppercase"
                    | "Lowercase"
                    | "Capitalize"
                    | "Uncapitalize"
                    | "NoInfer"
                    | "Awaited" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Blue),

                    // Boolean and null values
                    "true" | "false" | "null" => nu_ansi_term::Style::new()
                        .fg(nu_ansi_term::Color::Red)
                        .bold(),

                    // Built-in objects and global functions
                    "Andromeda" | "Event" | "OffscreenCanvas" | "console" | "Math" | "Date"
                    | "Array" | "Object" | "String" | "Number" | "Boolean" | "RegExp" | "Error"
                    | "Promise" | "JSON" | "parseInt" | "parseFloat" | "isNaN" | "isFinite"
                    | "encodeURIComponent" | "decodeURIComponent" | "Map" | "Set" | "WeakMap"
                    | "WeakSet" | "ArrayBuffer" | "DataView" | "Int8Array" | "Uint8Array"
                    | "Int16Array" | "Uint16Array" | "Int32Array" | "Uint32Array"
                    | "Float32Array" | "Float64Array" | "BigInt64Array" | "BigUint64Array" => {
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Magenta)
                    }

                    // Check if it's a type annotation context (follows a colon)
                    _ => {
                        // Look backwards to see if this word follows a colon (type annotation)
                        let mut j = word_start.saturating_sub(1);
                        while j > 0 && chars[j].is_whitespace() {
                            j = j.saturating_sub(1);
                        }
                        if j > 0 && chars[j] == ':' {
                            // This is likely a type annotation
                            nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Blue)
                        } else {
                            // Check if it looks like a type (PascalCase)
                            if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                                nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Blue)
                            } else {
                                nu_ansi_term::Style::new()
                            }
                        }
                    }
                };

                styled.push((style, word));
                continue;
            }

            // Handle multi-character TypeScript operators
            if ch == '=' && i + 1 < chars.len() && chars[i + 1] == '>' {
                // Arrow function
                styled.push((
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Purple),
                    "=>".to_string(),
                ));
                i += 2;
                continue;
            }

            if ch == '?' && i + 1 < chars.len() {
                if chars[i + 1] == '?' {
                    // Nullish coalescing operator
                    styled.push((
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red),
                        "??".to_string(),
                    ));
                    i += 2;
                    continue;
                } else if chars[i + 1] == '.' {
                    // Optional chaining
                    styled.push((
                        nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red),
                        "?.".to_string(),
                    ));
                    i += 2;
                    continue;
                }
            }

            if ch == '!' && i + 1 < chars.len() && chars[i + 1] == '!' {
                // Non-null assertion (TypeScript)
                styled.push((
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red),
                    "!!".to_string(),
                ));
                i += 2;
                continue;
            }

            // Handle pipe for union types
            if ch == '|' && i > 0 && i + 1 < chars.len() {
                // Check if this is a union type (surrounded by type context)
                let prev_non_space = chars[..i].iter().rposition(|&c| !c.is_whitespace());
                let next_non_space = chars[i + 1..]
                    .iter()
                    .position(|&c| !c.is_whitespace())
                    .map(|pos| i + 1 + pos);

                let is_union_type = match (prev_non_space, next_non_space) {
                    (Some(prev_idx), Some(next_idx)) => {
                        // Simple heuristic: if surrounded by alphanumeric characters or angle brackets, likely a union type
                        (chars[prev_idx].is_alphanumeric()
                            || chars[prev_idx] == '>'
                            || chars[prev_idx] == ']')
                            && (chars[next_idx].is_alphanumeric()
                                || chars[next_idx] == '<'
                                || chars[next_idx] == '('
                                || chars[next_idx] == '{')
                    }
                    _ => false,
                };

                let style = if is_union_type {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan)
                } else {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red) // Bitwise OR
                };
                styled.push((style, ch.to_string()));
                i += 1;
                continue;
            }

            // Handle ampersand for intersection types
            if ch == '&' && i > 0 && i + 1 < chars.len() && chars[i + 1] != '&' {
                // Similar heuristic for intersection types
                let prev_non_space = chars[..i].iter().rposition(|&c| !c.is_whitespace());
                let next_non_space = chars[i + 1..]
                    .iter()
                    .position(|&c| !c.is_whitespace())
                    .map(|pos| i + 1 + pos);

                let is_intersection_type = match (prev_non_space, next_non_space) {
                    (Some(prev_idx), Some(next_idx)) => {
                        (chars[prev_idx].is_alphanumeric()
                            || chars[prev_idx] == '>'
                            || chars[prev_idx] == ']')
                            && (chars[next_idx].is_alphanumeric()
                                || chars[next_idx] == '<'
                                || chars[next_idx] == '('
                                || chars[next_idx] == '{')
                    }
                    _ => false,
                };

                let style = if is_intersection_type {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan)
                } else {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red) // Bitwise AND
                };
                styled.push((style, ch.to_string()));
                i += 1;
                continue;
            }

            let style = match ch {
                '(' | ')' | '[' | ']' | '{' | '}' => {
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Yellow)
                }
                '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~'
                | '?' => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red),
                ';' | ',' | '.' => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::DarkGray),
                ':' => {
                    // TypeScript type annotations
                    nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan)
                }
                _ => nu_ansi_term::Style::new(),
            };

            styled.push((style, ch.to_string()));
            i += 1;
        }

        styled
    }
}
