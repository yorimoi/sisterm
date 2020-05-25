use std::collections::HashMap;
use std::io::Write;

use crate::setting;

use lazy_static::lazy_static;


//  Predefined colors
lazy_static! {
    static ref PREDEFINED_COLORS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("RESET",   "\x1b[0m");
        m.insert("BLACK",   "\x1b[30m");
        m.insert("RED",     "\x1b[31m");
        m.insert("GREEN",   "\x1b[32m");
        m.insert("YELLOW",  "\x1b[33m");
        m.insert("BLUE",    "\x1b[34m");
        m.insert("MAGENTA", "\x1b[35m");
        m.insert("CYAN",    "\x1b[36m");
        m.insert("WHITE",   "\x1b[37m");
        m
    };
}

pub fn coloring_from_file(text: String, params: Option<setting::Params>) {
    if params.is_none() {
        println!("{}", text);
        return;
    }

    let params = params.unwrap();
    let mut all_string = String::new();

    for line in text.lines() {
        let mut line_str = String::new();
        let mut increasing_str         = String::new();
        let mut prev_matched   = false;
        let mut substring_len  = 0;

        'outer: for c in line.chars() {
            increasing_str.push(c);
            for (index, syntax) in params.syntaxes.iter().enumerate() {
                if let Some(cap) = syntax.regex().captures(&increasing_str) {
                    if prev_matched {
                        let len = cap.get(0).unwrap().as_str().len();
                        if substring_len == len {
                            prev_matched = false;
                            line_str.push_str(PREDEFINED_COLORS["RESET"]);
                            increasing_str.clear();
                        } else {
                            substring_len = len;
                        }
                        line_str.push(c);
                    } else {
                        prev_matched = true;
                        let substr = cap.get(0).unwrap().as_str();
                        let len = substr.len();
                        let color = params.syntaxes[index as usize].color();
                        line_str.push_str(&increasing_str[..increasing_str.len()-len]);
                        line_str.push_str(&color);
                        line_str.push_str(&substr);
                        increasing_str = substr.to_string();
                        substring_len = len;
                    }
                    continue 'outer;
                }
            }

            if prev_matched {
                prev_matched = false;
                line_str.push_str(PREDEFINED_COLORS["RESET"]);
                line_str.push_str(&increasing_str);
                increasing_str.clear();
            }
        }

        if !prev_matched {
            line_str.push_str(&increasing_str);
        }
        line_str.push_str(PREDEFINED_COLORS["RESET"]);
        all_string.push_str(&line_str);
        all_string.push('\n');
    }

    println!("{}", all_string);
}

pub fn coloring_words(serial_buf: &str, (word, colored): &mut (String, bool), params: &Option<setting::Params>) {
    if let Some(params) = params {
        let mut all_string = String::new();

        for c in serial_buf.chars() {
            let mut matched = false;
            let mut index: usize = 0;

            if c == '\r' || c == '\n' {
                word.clear();
                if *colored {
                    all_string.push_str(PREDEFINED_COLORS["RESET"]);
                }
                all_string.push(c);
                *colored = false;
                continue;
            } else {
                if word.ends_with(' ') {
                    if c != ' ' {
                        word.clear();
                        if *colored {
                            all_string.push_str(PREDEFINED_COLORS["RESET"]);
                        }
                        *colored = false;
                    }
                } else if c == ' ' {
                    word.clear();
                    if *colored {
                        all_string.push_str(PREDEFINED_COLORS["RESET"]);
                    }
                    *colored = false;
                }
                word.push(c);
            }

            for (i, syntax) in params.syntaxes.iter().enumerate() {
                if syntax.regex().captures(&word).is_some() {
                    matched = true;
                    index = i;
                    break;
                }
            }

            if matched {
                let color = params.syntaxes[index].color();  // assert Some()
                if *colored {
                    all_string.push(c);
                } else {
                    all_string += &format!("{:\x08<1$}{2}{3}", "", word.len()-1, color, word);
                    *colored = true;
                }

            } else {
                if *colored {
                    all_string.push_str(PREDEFINED_COLORS["RESET"]);
                }
                all_string.push(c);
                *colored = false;
            }
        }

        std::io::stdout().write_all(all_string.as_bytes()).unwrap();
    }
}

/* Color example
    * RED
    * 001
    * FF0000
    * (255, 0, 0)
 */
pub fn valid_color_syntax(coloring: &setting::Coloring) -> Result<String, String> {
    let color      = coloring.color();
    let underlined = coloring.underlined();

    if color.is_empty() {
        if underlined {
            return Ok("\x1b[4m".to_string());
        } else {
            return Ok("\x1b[0m".to_string());
        }
    }
    if is_predefined_color(&color) {
        if underlined {
            return Ok(format!("\x1b[4m{}", PREDEFINED_COLORS[color]));
        } else {
            return Ok(PREDEFINED_COLORS[color].to_string());
        }
    }
    if is_8bit_color(&color) {
        return Ok(to_8bit_color(&color, underlined));
    }
    if is_24bit_color(&color) {
        return Ok(to_24bit_color(&color, underlined));
    }
    if is_rgb_color(&color) {
        return Ok(to_rgb_color(&color, underlined));
    }

    Err(format!("invalid color syntax: \"{}\"", color))
}

fn is_predefined_color(color: &str) -> bool {
    PREDEFINED_COLORS.get(color).is_some()
}

fn is_8bit_color(color: &str) -> bool {
    color.parse::<u8>().is_ok()
}

fn is_24bit_color(color: &str) -> bool {
    color.len() == 6 && i32::from_str_radix(color, 16).is_ok()
}

fn is_rgb_color(color: &str) -> bool {
    if !color.starts_with('(') || !color.ends_with(')') {
        return false;
    }
    let color = &color[1..color.len()-1].replace(',', " ");
    let rgb: Vec<&str> = color.split_whitespace().collect();
    if rgb.len() != 3 {
        return false;
    }
    for color in rgb {
        if color.parse::<u8>().is_err() {
            return false;
        }
    }

    true
}

fn to_8bit_color(color: &str, underlined: bool) -> String {
    if underlined {
        format!("\x1b[4;38;5;{}m", color)
    } else {
        format!("\x1b[38;5;{}m", color)
    }
}

fn to_24bit_color(color: &str, underlined: bool) -> String {
    let r: u8 = u8::from_str_radix(&color[..2],  16).unwrap();
    let g: u8 = u8::from_str_radix(&color[2..4], 16).unwrap();
    let b: u8 = u8::from_str_radix(&color[4..],  16).unwrap();

    if underlined {
        format!("\x1b[4;38;2;{};{};{}m", r, g, b)
    } else {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    }
}

fn to_rgb_color(color: &str, underlined: bool) -> String {
    let color = &color[1..color.len()-1].replace(',', " ");
    let rgb: Vec<&str> = color.split_whitespace().collect();

    if underlined {
        format!("\x1b[4;38;2;{};{};{}m", rgb[0], rgb[1], rgb[2])
    } else {
        format!("\x1b[38;2;{};{};{}m", rgb[0], rgb[1], rgb[2])
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_predefined_color() {
        let tests = vec![
            (
                "BLACK",
                true,
            ),
            (
                "black",
                false,
            ),
            (
                "",
                false,
            ),
            (
                "shiro",
                false,
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(is_predefined_color(input), expect);
        }
    }

    #[test]
    fn test_is_8bit_color() {
        let tests = vec![
            (
                "000",
                true,
            ),
            (
                "001",
                true,
            ),
            (
                "255",
                true,
            ),
            (
                "0",
                true,
            ),
            (
                "10",
                true,
            ),
            (
                "  1",
                false,
            ),
            (
                "256",
                false,
            ),
            (
                "aaa",
                false,
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(is_8bit_color(input), expect);
        }
    }

    #[test]
    fn test_is_24bit_color() {
        let tests = vec![
            (
                "000000",
                true,
            ),
            (
                "FF0000",
                true,
            ),
            (
                "FFFFFF",
                true,
            ),
            (
                "abcdef",
                true,
            ),
            (
                "GGGGGG",
                false,
            ),
            (
                "ff000",
                false,
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(is_24bit_color(input), expect);
        }
    }

    #[test]
    fn test_is_rgb_color() {
        let tests = vec![
            (
                "(0, 0, 0)",
                true,
            ),
            (
                "(000, 000, 000)",
                true,
            ),
            (
                "(255, 0, 0)",
                true,
            ),
            (
                "(255, 255, 255)",
                true,
            ),
            (
                "(255 255 255)",
                true,
            ),
            (
                "(256, 255, 255)",
                false,
            ),
            (
                "(FF, FF, FF)",
                false,
            ),
            (
                "(0, 0, 0, 0)",
                false,
            ),
            (
                "[255, 255, 255]",
                false,
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(is_rgb_color(input), expect);
        }
    }

    #[test]
    fn test_to_8bit_color() {
        let tests = vec![
            (
                "000",
                "\x1b[38;5;000m",
            ),
            (
                "001",
                "\x1b[38;5;001m",
            ),
            (
                "255",
                "\x1b[38;5;255m",
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(to_8bit_color(input, false), expect);
        }
    }

    #[test]
    fn test_to_24bit_color() {
        let tests = vec![
            (
                "000000",
                "\x1b[38;2;0;0;0m",
            ),
            (
                "FF0000",
                "\x1b[38;2;255;0;0m",
            ),
            (
                "FFFFFF",
                "\x1b[38;2;255;255;255m",
            ),
            (
                "abcdef",
                "\x1b[38;2;171;205;239m",
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(to_24bit_color(input, false), expect);
        }
    }

    #[test]
    fn test_to_rgb_color() {
        let tests = vec![
            (
                "(0, 0, 0)",
                "\x1b[38;2;0;0;0m",
            ),
            (
                "(000, 000, 000)",
                "\x1b[38;2;000;000;000m",
            ),
            (
                "(255, 0, 0)",
                "\x1b[38;2;255;0;0m",
            ),
            (
                "(255, 255, 255)",
                "\x1b[38;2;255;255;255m",
            ),
            (
                "(255 255 255)",
                "\x1b[38;2;255;255;255m",
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(to_rgb_color(input, false), expect);
        }
    }

    #[test]
    fn test_underlined() {
        let tests = vec![
            (
                to_8bit_color("123", true),
                "\x1b[4;38;5;123m",
            ),
            (
                to_24bit_color("000000", true),
                "\x1b[4;38;2;0;0;0m",
            ),
            (
                to_rgb_color("(0, 0, 0)", true),
                "\x1b[4;38;2;0;0;0m",
            ),
        ];

        for (actual, expect) in tests {
            assert_eq!(actual, expect);
        }
    }
}
