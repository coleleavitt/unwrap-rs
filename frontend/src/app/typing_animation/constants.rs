use rand::{Rng, RngExt};

pub const EXTRA_PARTICLES: usize = 80;
pub const TARGET_TEXT: &str = ".unwrap()";

/// Layout units per character. Particles render at `UNIT_EM` em per unit, so
/// one advance equals the site font's 0.6em glyph width and `.unwrap()` sets flush
/// against the static `Result<T, E>` at any font size.
pub const CHAR_ADVANCE: f32 = 20.0;
pub const UNIT_EM: f32 = 0.6 / CHAR_ADVANCE;

/// Field size in layout units: the resolved text plus one advance of slack.
pub const WIDTH: f32 = (TARGET_TEXT.len() + 1) as f32 * CHAR_ADVANCE;
pub const HEIGHT: f32 = 100.0;

/// Center of the first glyph, so the leading `.` touches the preceding `>`.
pub const TEXT_LEFT: f32 = CHAR_ADVANCE / 2.0;
pub const TEXT_Y: f32 = HEIGHT / 2.0;
pub const TEXT_CENTER_X: f32 = TEXT_LEFT + (TARGET_TEXT.len() - 1) as f32 * CHAR_ADVANCE / 2.0;

const SYMBOL_CHARS: &str = "⏃ᗷᑕᗞ⟊⎎Ꮆ⋔⟙⟗Ꮶ⅃⏁ᑎ〇℘Ϙ尺⟅ナ⋒٧山〤Ꭹㄗค♭ᑢↁ⋿ℱᎶЂ|ן⊗≠ፐ∪∨พу⊼⋕⦿|ᒿ≡⫓⫔⏀⫛∞ⴤ•⦑⦒⎼⊕⧠";

pub fn random_symbol(rng: &mut impl Rng) -> char {
    let index = rng.random_range(0..SYMBOL_CHARS.len());
    SYMBOL_CHARS.chars().nth(index).unwrap_or('⧠')
}

pub const fn map_char(c: char) -> char {
    match c {
        'A' => '⏃',
        'B' => 'ᗷ',
        'C' => 'ᑕ',
        'D' => 'ᗞ',
        'E' => '⟊',
        'F' => '⎎',
        'G' => 'Ꮆ',
        'H' => '⋔',
        'I' => '⟙',
        'J' => '⟗',
        'K' => 'Ꮶ',
        'L' => '⅃',
        'M' => '⏁',
        'N' => 'ᑎ',
        'O' => '〇',
        'P' => '℘',
        'Q' => 'Ϙ',
        'R' => '尺',
        'S' => '⟅',
        'T' => 'ナ',
        'U' => '⋒',
        'V' => '٧',
        'W' => '山',
        'X' => '〤',
        'Y' => 'Ꭹ',
        'Z' => 'ㄗ',
        'a' => 'ค',
        'b' => '♭',
        'c' => 'ᑢ',
        'd' => 'ↁ',
        'e' => '⋿',
        'f' => 'ℱ',
        'g' => 'Ꮆ',
        'h' => 'Ђ',
        'i' => '|',
        'j' => 'ן',
        'k' => 'к',
        'l' => '↳',
        'm' => '൩',
        'n' => 'ภ',
        'o' => '◯',
        'p' => '℘',
        'q' => 'զ',
        'r' => 'г',
        's' => '≠',
        't' => 'ፐ',
        'u' => '∪',
        'v' => '∨',
        'w' => 'พ',
        'x' => '⊗',
        'y' => 'у',
        'z' => 'չ',
        '0' => '⦿',
        '1' => '|',
        '2' => 'ᒿ',
        '3' => '≡',
        '4' => '⫓',
        '5' => '⫔',
        '6' => '⏀',
        '7' => '⫛',
        '8' => '∞',
        '9' => 'ⴤ',
        '.' => '•',
        '(' => '⦑',
        ')' => '⦒',
        '_' => '⎼',
        '!' => '⫝',
        '@' => '⊛',
        '#' => '⧇',
        '$' => '⧫',
        '%' => '⧮',
        '^' => '△',
        '&' => '⊼',
        '*' => '⋆',
        '-' => '⎯',
        '+' => '⊕',
        '=' => '⋕',
        '{' => '⦓',
        '}' => '⦔',
        '[' => '【',
        ']' => '】',
        '|' => '┃',
        '\\' => '⧹',
        ':' => '∴',
        ';' => '⁏',
        '\'' => '´',
        '"' => '‶',
        ',' => '⸴',
        '<' => '⫷',
        '>' => '⫸',
        '/' => '⧸',
        '?' => '⸮',
        '`' => '῾',
        '~' => '∿',
        _ => '⧠',
    }
}
