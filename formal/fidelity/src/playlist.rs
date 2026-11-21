//! Generates random operation scripts for the playlist target.

/// `SplitMix64`: small, deterministic, and good enough to pick test inputs.
#[derive(Debug)]
pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `low..=high`.
    fn range(&mut self, low: i64, high: i64) -> i64 {
        let span = u64::try_from(high - low + 1).unwrap_or(1);
        low + i64::try_from(self.next() % span).unwrap_or(0)
    }

    fn direction(&mut self) -> i64 {
        if self.next().is_multiple_of(2) { -1 } else { 1 }
    }
}

/// One script line per operation. Every operation respects the C function's
/// asserted preconditions, so the real code never hits undefined behaviour.
pub(crate) fn script(rng: &mut Rng, sequences: usize, operations: usize) -> String {
    let mut out = String::new();
    for _ in 0..sequences {
        out.push_str("reset\n");
        let mut len = 0_i64;
        for _ in 0..operations {
            let last = len - 1;
            let line = match rng.range(0, 13) {
                // Keep playlists short so boundaries are hit often.
                0..=2 if len < 6 => {
                    len += 1;
                    format!("insert {}", rng.range(-1, last))
                }
                3 | 4 if len > 0 => {
                    len -= 1;
                    format!("remove {}", rng.range(0, last))
                }
                5 => format!("setcur {}", rng.range(-1, last)),
                6 => format!(
                    "flags {} {} {}",
                    rng.range(0, 1),
                    rng.range(0, 1),
                    rng.range(0, 1)
                ),
                7 => "count".to_owned(),
                8 => format!("fromidx {}", rng.range(-2, len + 1)),
                9 => format!("toidx {}", rng.range(-2, last)),
                10 if rng.next().is_multiple_of(2) => "first".to_owned(),
                10 => "last".to_owned(),
                11 => {
                    let entry = if len == 0 || rng.next().is_multiple_of(8) {
                        -2
                    } else {
                        rng.range(0, last)
                    };
                    format!("rel {entry} {}", rng.direction())
                }
                12 => format!(
                    "reindex {} {} {} {}",
                    rng.range(-1, last),
                    rng.range(-3, len + 3),
                    rng.range(-3, len + 3),
                    rng.range(-3, len + 3)
                ),
                _ => format!("next {}", rng.direction()),
            };
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}
