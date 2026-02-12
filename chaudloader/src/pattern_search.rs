#[derive(Debug, Clone, Copy)]
pub struct SearchPattern<const N: usize> {
    pub bytes: [u8; N],
    pub mask: [u8; N],
}

impl<const N: usize> SearchPattern<N> {
    /// Constructs the pattern. This is a const fn, allowing it to run at compile time.
    pub const fn new(pattern_str: &str) -> Self {
        let input = pattern_str.as_bytes();
        let mut bytes = [0u8; N];
        let mut mask = [0u8; N];

        let mut out_idx = 0;
        let mut i = 0;

        while i < input.len() {
            if input[i] == b' ' {
                i += 1;
                continue;
            }

            if input[i] == b'?' {
                // Wildcard
                bytes[out_idx] = 0x00;
                mask[out_idx] = 0x00;

                // Skip next char if it is also '?' (handling "??")
                if i + 1 < input.len() && input[i + 1] == b'?' {
                    i += 2;
                } else {
                    i += 1;
                }
            } else {
                // Hex Byte
                let high = char_to_int(input[i]);
                // Safety: C++ code assumes valid hex pairs if not wildcard.
                // We add a check or assume valid input for const context.
                let low = if i + 1 < input.len() {
                    char_to_int(input[i + 1])
                } else {
                    0
                };

                bytes[out_idx] = (high << 4) | low;
                mask[out_idx] = 0xFF;
                i += 2;
            }
            out_idx += 1;
        }

        Self { bytes, mask }
    }
}

const fn char_to_int(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'A'..=b'F' => c - b'A' + 0x0A,
        b'a'..=b'f' => c - b'a' + 0x0A,
        _ => 0,
    }
}

pub const fn count_pattern_bytes(input: &[u8]) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < input.len() {
        if input[i] == b' ' {
            i += 1;
            continue;
        }
        count += 1;
        if input[i] == b'?' {
            if i + 1 < input.len() && input[i + 1] == b'?' {
                i += 2;
            } else {
                i += 1;
            }
        } else {
            i += 2;
        }
    }
    count
}

#[macro_export]
macro_rules! make_pattern {
    ($s:literal) => {{
        const P: pattern_search::SearchPattern<
            { pattern_search::count_pattern_bytes($s.as_bytes()) },
        > = pattern_search::SearchPattern::new($s);
        P
    }};
}

pub fn find_pattern<const N: usize>(data: &[u8], pattern: &SearchPattern<N>) -> Option<usize> {
    data.windows(N).position(|window| {
        window
            .iter()
            .zip(&pattern.bytes)
            .zip(&pattern.mask)
            .all(|((&mem, &pat), &mask)| (mem ^ pat) & mask == 0)
    })
}
