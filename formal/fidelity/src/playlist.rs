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

    fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        let index = usize::try_from(self.next()).unwrap_or(0) % items.len().max(1);
        items.get(index).copied()
    }
}

/// Filenames for `playlist_entry_new`, each with the filename mpv stores for it.
/// Relative paths and `..` are left out: their result depends on the working
/// directory and the filesystem.
const FILES: [(&str, &str); 12] = [
    ("/a", "/a"),
    ("/b", "/b"),
    ("/./a", "/a"),
    ("/a//b/./c/", "/a/b/c"),
    ("//", "/"),
    ("-", "-"),
    ("http://h/x", "http://h/x"),
    ("file:///b", "/b"),
    ("file:///a%20b", "/a b"),
    ("FILE:///a/./b//c", "/a/b/c"),
    ("file://%2Fa", "/a"),
    ("file:///a%zzb%4", "/a%zzb%4"),
];

/// Playlist paths; most equal a stored filename so `playlist_set_current` matches.
/// Some are a prefix of another, with and without a trailing separator, for
/// `playlist_get_first_in_same_playlist`.
const PATHS: [&str; 8] = [
    "/a",
    "/b",
    "/a/b/c",
    "/",
    "http://h/x",
    "/other",
    "/a/",
    "/a/b",
];

/// Playlist files for `playlist_parse_file`. `mp_get_user_path` expands the second.
const PLAYLIST_FILES: [&str; 2] = ["/lists/one", "~/two"];

/// Most playlists stay shorter than this so boundaries are hit often.
const SHORT: i64 = 6;
/// Transfers may grow a playlist up to this.
const LONGEST: i64 = 9;

#[derive(Debug)]
struct Entry {
    token: u64,
    filename: &'static str,
    playlist_path: Option<&'static str>,
    /// C `original_index`: `-1` until the playlist is shuffled.
    original_index: i64,
}

/// The entry the harness holds a reference on.
#[derive(Debug)]
struct Held {
    token: u64,
    reserved: i64,
    listed: bool,
}

/// C `playlist_completed` and `playlist_started`, which a transfer copies.
#[derive(Clone, Copy, Debug, Default)]
struct Progress {
    completed: bool,
    started: bool,
}

/// What the generator must know about the playlist to stay within the C
/// functions' preconditions.
#[derive(Debug, Default)]
struct Shadow {
    entries: Vec<Entry>,
    current: Option<u64>,
    current_was_replaced: bool,
    progress: Progress,
    has_dir: bool,
    held: Option<Held>,
    tokens: u64,
    /// The second playlist, which the transfer functions empty into the first.
    source: Vec<Entry>,
    source_progress: Progress,
}

/// `playlist_shuffle` with the harness's replacement for `mp_rand_in_range32`.
fn shuffle(entries: &mut [Entry], numbers: &[i64]) {
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.original_index = i64::try_from(index).unwrap_or(0);
    }
    let len = entries.len();
    for (index, number) in numbers.iter().enumerate().take(len.saturating_sub(1)) {
        let pick = index + usize::try_from(*number).unwrap_or(0) % (len - index);
        entries.swap(index, pick);
    }
}

/// One number per call `playlist_shuffle` makes to `mp_rand_in_range32`.
fn shuffle_numbers(rng: &mut Rng, len: usize) -> (Vec<i64>, String) {
    let numbers: Vec<i64> = (1..len).map(|_| rng.range(0, 99)).collect();
    let words: Vec<String> = numbers.iter().map(|number| format!(" {number}")).collect();
    let text = format!("{}{}", numbers.len(), words.concat());
    (numbers, text)
}

impl Shadow {
    fn len(&self) -> i64 {
        i64::try_from(self.entries.len()).unwrap_or(0)
    }

    fn token_at(&self, index: i64) -> Option<u64> {
        let index = usize::try_from(index).ok()?;
        self.entries.get(index).map(|entry| entry.token)
    }

    fn source_len(&self) -> i64 {
        i64::try_from(self.source.len()).unwrap_or(0)
    }

    fn new_entry(&mut self, filename: &'static str) -> Entry {
        self.tokens += 1;
        Entry {
            token: self.tokens,
            filename,
            playlist_path: None,
            original_index: -1,
        }
    }

    fn insert(&mut self, at: i64, filename: &'static str) {
        let entry = self.new_entry(filename);
        let index = usize::try_from(at).unwrap_or(self.entries.len());
        self.entries.insert(index.min(self.entries.len()), entry);
    }

    fn source_insert(&mut self, at: i64, filename: &'static str) {
        let entry = self.new_entry(filename);
        let index = usize::try_from(at).unwrap_or(self.source.len());
        self.source.insert(index.min(self.source.len()), entry);
    }

    fn current_index(&self) -> Option<usize> {
        let current = self.current?;
        self.entries.iter().position(|entry| entry.token == current)
    }

    /// libc defines `qsort` only for a consistent order. `cmp_unshuffle` is one
    /// exactly when no entry with an unknown original index follows one with a
    /// known index.
    fn can_unshuffle(&self) -> bool {
        self.entries
            .iter()
            .skip_while(|entry| entry.original_index < 0)
            .all(|entry| entry.original_index >= 0)
    }

    /// `playlist_get_first_in_next_playlist` reads `pl->current->playlist_path`
    /// without checking `pl->current`. This is false when that would crash.
    fn can_next_playlist(&self, direction: i64) -> bool {
        if self.current.is_some() || self.entries.is_empty() {
            return true;
        }
        if self.progress.completed && direction < 0 {
            return false;
        }
        if !self.progress.started && direction > 0 {
            return self
                .entries
                .first()
                .is_some_and(|entry| entry.playlist_path.is_none());
        }
        true
    }

    /// Where `playlist_transfer_entries` inserts, when its assertions hold.
    fn transfer_index(&self) -> Option<usize> {
        let Some(current) = self.current_index() else {
            return Some(self.entries.len());
        };
        let index = current + 1 + usize::from(self.current_was_replaced);
        (index <= self.entries.len()).then_some(index)
    }

    fn drop_source(&mut self) {
        self.source.clear();
        self.source_progress = Progress::default();
    }

    fn transfer(&mut self, at: usize) {
        let moved = std::mem::take(&mut self.source);
        let at = at.min(self.entries.len());
        self.entries.splice(at..at, moved);
        self.progress = self.source_progress;
        self.drop_source();
    }

    fn remove(&mut self, index: usize) {
        if index >= self.entries.len() {
            return;
        }
        let entry = self.entries.remove(index);
        if self.current == Some(entry.token) {
            self.current = self.entries.get(index).map(|next| next.token);
            self.current_was_replaced = true;
        }
        if let Some(held) = self.held.as_mut()
            && held.token == entry.token
        {
            held.listed = false;
            held.reserved -= 1;
        }
    }

    fn clear(&mut self, keep_current: bool) {
        for index in (0..self.entries.len()).rev() {
            let token = self.entries.get(index).map(|entry| entry.token);
            if !(keep_current && token == self.current) {
                self.remove(index);
            }
        }
        self.progress = Progress::default();
        if !keep_current {
            self.current_was_replaced = false;
            self.has_dir = false;
        }
    }

    fn move_entry(&mut self, from: i64, at: i64) {
        let (Ok(from), len) = (usize::try_from(from), self.entries.len()) else {
            return;
        };
        let index = usize::try_from(at).unwrap_or(len);
        if from == index || from >= len {
            return;
        }
        let entry = self.entries.remove(from);
        let target = if from < index { index - 1 } else { index };
        self.entries.insert(target, entry);
    }

    fn set_current(&mut self) {
        if !self.has_dir {
            return;
        }
        let found = self
            .entries
            .iter()
            .find(|entry| entry.playlist_path == Some(entry.filename));
        if let Some(entry) = found {
            self.current = Some(entry.token);
        }
    }

    /// `playlist_entry_unref` may run when it frees nothing or the entry is detached.
    fn can_unref(&self) -> bool {
        self.held
            .as_ref()
            .is_some_and(|held| held.reserved >= 1 || !held.listed)
    }

    fn unref(&mut self) {
        let Some(held) = self.held.as_mut() else {
            return;
        };
        held.reserved -= 1;
        // Freed, or owned by the playlist alone again.
        if held.reserved < 0 || (held.listed && held.reserved == 0) {
            self.held = None;
        }
    }
}

/// A count and that many name and value words.
fn params(rng: &mut Rng) -> String {
    let count = rng.range(0, 3);
    let pairs: Vec<String> = (0..count)
        .map(|_| format!(" n{} v{}", rng.range(0, 2), rng.range(0, 2)))
        .collect();
    format!("{count}{}", pairs.concat())
}

/// One script line per operation. Every operation respects the C function's
/// asserted preconditions, so the real code never hits undefined behaviour.
pub(crate) fn script(rng: &mut Rng, sequences: usize, operations: usize) -> String {
    let mut out = String::new();
    for _ in 0..sequences {
        out.push_str("reset\n");
        let mut shadow = Shadow::default();
        for _ in 0..operations {
            out.push_str(&operation(rng, &mut shadow));
            out.push('\n');
        }
    }
    out
}

#[expect(
    clippy::too_many_lines,
    reason = "one arm per script operation; splitting the match would hide the weights"
)]
fn operation(rng: &mut Rng, shadow: &mut Shadow) -> String {
    let len = shadow.len();
    let last = len - 1;
    let (file, stored) = rng.pick(&FILES).unwrap_or(("/a", "/a"));
    match rng.range(0, 48) {
        0..=3 if len < SHORT => {
            let at = rng.range(-1, last);
            shadow.insert(at, stored);
            format!("insert {at} {file}")
        }
        4 | 5 if len < SHORT => {
            shadow.insert(-1, stored);
            format!("append {file}")
        }
        6 | 7 if len > 0 => {
            let index = rng.range(0, last);
            shadow.remove(usize::try_from(index).unwrap_or(0));
            format!("remove {index}")
        }
        8 if rng.next().is_multiple_of(2) => {
            shadow.clear(false);
            "clear".to_owned()
        }
        9 if rng.next().is_multiple_of(2) => {
            shadow.clear(true);
            "clearx".to_owned()
        }
        10..=12 if len > 0 => {
            let (from, at) = (rng.range(0, last), rng.range(-1, last));
            shadow.move_entry(from, at);
            format!("move {from} {at}")
        }
        13 => {
            let index = rng.range(-1, last);
            shadow.current = shadow.token_at(index);
            format!("setcur {index}")
        }
        14 => {
            shadow.current_was_replaced = rng.next().is_multiple_of(2);
            shadow.progress = Progress {
                completed: rng.next().is_multiple_of(2),
                started: rng.next().is_multiple_of(2),
            };
            format!(
                "flags {} {} {}",
                i32::from(shadow.current_was_replaced),
                i32::from(shadow.progress.completed),
                i32::from(shadow.progress.started)
            )
        }
        15 => {
            shadow.has_dir = rng.next().is_multiple_of(2);
            format!("setdir {}", i32::from(shadow.has_dir))
        }
        16 => {
            let path = rng.pick(&PATHS).unwrap_or("/a");
            for entry in &mut shadow.entries {
                entry.playlist_path = Some(path);
            }
            format!("populate {path}")
        }
        17 => format!("streamflags {}", rng.range(0, 31)),
        18 if len > 0 => format!(
            "addparam {} n{} v{}",
            rng.range(0, last),
            rng.range(0, 2),
            rng.range(0, 2)
        ),
        19 if len > 0 => format!("addparams {} {}", rng.range(0, last), params(rng)),
        20 => format!("setparams {}", params(rng)),
        21 | 22 => {
            shadow.set_current();
            "setcurrent".to_owned()
        }
        23 if len > 0 && shadow.held.is_none() => {
            let index = rng.range(0, last);
            shadow.held = shadow.token_at(index).map(|token| Held {
                token,
                reserved: 1,
                listed: true,
            });
            format!("hold {index}")
        }
        23 | 24 if shadow.held.as_ref().is_some_and(|held| held.reserved < 2) => {
            if let Some(held) = shadow.held.as_mut() {
                held.reserved += 1;
            }
            "ref".to_owned()
        }
        24 | 25 if shadow.can_unref() => {
            shadow.unref();
            "unref".to_owned()
        }
        26 => format!("new {file}"),
        27 => "count".to_owned(),
        28 => format!("fromidx {}", rng.range(-2, len + 1)),
        29 if shadow.held.is_some() && rng.next().is_multiple_of(2) => "toidx -3".to_owned(),
        29 => format!("toidx {}", rng.range(-2, last)),
        30 if rng.next().is_multiple_of(2) => "first".to_owned(),
        30 => "last".to_owned(),
        31 if rng.next().is_multiple_of(2) => {
            let entry = if shadow.held.is_some() && rng.next().is_multiple_of(4) {
                -3
            } else if len == 0 || rng.next().is_multiple_of(8) {
                -2
            } else {
                rng.range(0, last)
            };
            format!("rel {entry} {}", rng.direction())
        }
        31 => format!(
            "reindex {} {} {} {}",
            rng.range(-1, last),
            rng.range(-3, len + 3),
            rng.range(-3, len + 3),
            rng.range(-3, len + 3)
        ),
        32 | 33 => {
            let (numbers, text) = shuffle_numbers(rng, shadow.entries.len());
            shuffle(&mut shadow.entries, &numbers);
            format!("shuffle {text}")
        }
        34 | 35 if shadow.can_unshuffle() => {
            // Stable, so entries with equal original indexes keep their order.
            shadow.entries.sort_by_key(|entry| entry.original_index);
            "unshuffle".to_owned()
        }
        36 if len > 0 => format!("cmp {} {}", rng.range(0, last), rng.range(0, last)),
        37 | 38 => {
            let direction = rng.direction();
            if shadow.can_next_playlist(direction) {
                format!("nextpl {direction}")
            } else {
                format!("next {direction}")
            }
        }
        39 => {
            let entry = if shadow.held.is_some() && rng.next().is_multiple_of(8) {
                -3
            } else {
                rng.range(-2, last)
            };
            let path = if rng.next().is_multiple_of(8) {
                "~"
            } else {
                rng.pick(&PATHS).unwrap_or("/a")
            };
            format!("samepl {entry} {path}")
        }
        40 | 41 if shadow.source_len() < 4 => {
            let at = rng.range(-1, shadow.source_len() - 1);
            shadow.source_insert(at, stored);
            format!("srcinsert {at} {file}")
        }
        42 => {
            let path = rng.pick(&PATHS).unwrap_or("/a");
            for entry in &mut shadow.source {
                entry.playlist_path = Some(path);
            }
            format!("srcpopulate {path}")
        }
        43 if rng.next().is_multiple_of(2) => {
            shadow.source_progress = Progress {
                completed: rng.next().is_multiple_of(2),
                started: rng.next().is_multiple_of(2),
            };
            format!(
                "srcflags {} {}",
                i32::from(shadow.source_progress.completed),
                i32::from(shadow.source_progress.started)
            )
        }
        43 => format!("srcsetcur {}", rng.range(-1, shadow.source_len() - 1)),
        44 if rng.next().is_multiple_of(2) => {
            let (numbers, text) = shuffle_numbers(rng, shadow.source.len());
            shuffle(&mut shadow.source, &numbers);
            format!("srcshuffle {text}")
        }
        44 | 45 if len + shadow.source_len() <= LONGEST => {
            let at = rng.range(0, len);
            shadow.transfer(usize::try_from(at).unwrap_or(0));
            format!("transferto {at}")
        }
        46 if len + shadow.source_len() <= LONGEST && shadow.transfer_index().is_some() => {
            let at = shadow.transfer_index().unwrap_or(0);
            shadow.transfer(at);
            "transfer".to_owned()
        }
        47 if len + shadow.source_len() <= LONGEST => {
            shadow.transfer(shadow.entries.len());
            "appendentries".to_owned()
        }
        48 => {
            let file = rng.pick(&PLAYLIST_FILES).unwrap_or("/lists/one");
            let kind = rng.range(0, 2);
            let filetype = rng.pick(&["~", "hls", "m3u"]).unwrap_or("~");
            if kind == 2 {
                shadow.drop_source();
            }
            format!("parse {file} {kind} {filetype}")
        }
        _ => format!("next {}", rng.direction()),
    }
}
