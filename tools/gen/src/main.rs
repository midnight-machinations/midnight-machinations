//! Generators for files shared between the client and server.
//!
//! Run `mask gen <generator>` from the repository root. `mask config` runs `vars`.

use std::{collections::HashMap, fs, io, path::{Path, PathBuf}};

use serde_json::{Map, Value};

mod vars;

type Generator = (&'static str, fn(&Path) -> io::Result<()>);

const USAGE: &str = "Usage: gen <vars | dummy-names | broken-keyboard | scrambled | all>";

fn main() -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let generators: &[Generator] = &[
        ("dummy-names", dummy_names),
        ("broken-keyboard", broken_keyboard),
        ("scrambled", scrambled),
    ];
    let arg = std::env::args().nth(1).ok_or(USAGE)?;
    // Local settings are separate from the checked-in generated files, so `all` skips them.
    if arg == "vars" {
        return vars::generate(&root).map_err(|error| format!("vars: {error}"));
    }
    let selected: Vec<_> = generators.iter().filter(|(name, _)| arg == "all" || arg == *name).collect();
    if selected.is_empty() {
        return Err(USAGE.into());
    }
    for (name, generate) in selected {
        generate(&root).map_err(|error| format!("{name}: {error}"))?;
        println!("Generated {name}");
    }
    Ok(())
}

/// `assets/random_names/names.json` is the source for the client's dummy names
/// and the server's random name lists.
fn dummy_names(root: &Path) -> io::Result<()> {
    let names: Value = serde_json::from_str(&fs::read_to_string(root.join("assets/random_names/names.json"))?)?;
    let list = |key: &str| -> Vec<String> {
        names[key].as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).collect()
    };
    let (default_names, extra_names) = (list("defaultNames"), list("extraNames"));
    let combined: Vec<Value> = default_names.iter().chain(&extra_names).cloned().map(Value::String).collect();
    write_json(root.join("assets/resources/dummyNames.json"), &Value::Array(combined))?;
    // One name per line, quoted like Python's csv writer when needed.
    let csv = |names: &[String]| names.iter().map(|name| csv_field(name) + "\r\n").collect::<String>();
    fs::write(root.join("assets/random_names/default_names.csv"), csv(&default_names))?;
    fs::write(root.join("assets/random_names/extra_names.csv"), csv(&extra_names))
}

fn csv_field(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_owned()
    }
}

/// Removes vowels from every word.
fn broken_keyboard(root: &Path) -> io::Result<()> {
    transform_lang(root, "broken_keyboard.json", "Broken Keyboard", |_, value| {
        value.chars().filter(|c| !"aeiouAEIOU".contains(*c)).collect()
    })
}

/// Shuffles the letters of every word. Each word is shuffled once, the first time
/// it appears, so it reads the same everywhere.
fn scrambled(root: &Path) -> io::Result<()> {
    let mut cache: HashMap<String, Vec<char>> = HashMap::new();
    transform_lang(root, "dyslexic.json", "Scrambled", |key, value| {
        let mut rng = Rng::seeded(key);
        tokens(value)
            .map(|token| {
                if !token.chars().all(is_word_char) {
                    return token.to_owned();
                }
                let lower = token.to_lowercase();
                let shuffled = cache.entry(lower.clone()).or_insert_with(|| {
                    let mut letters: Vec<char> = lower.chars().collect();
                    rng.shuffle(&mut letters);
                    letters
                });
                let upper: Vec<bool> = token.chars().map(char::is_uppercase).collect();
                shuffled
                    .iter()
                    .enumerate()
                    .map(|(i, c)| if upper.get(i).copied().unwrap_or(false) { c.to_uppercase().to_string() } else { c.to_lowercase().to_string() })
                    .collect()
            })
            .collect()
    })
}

/// Rewrites every value of `en_us.json` into another client language file.
fn transform_lang(root: &Path, file: &str, language: &str, mut transform: impl FnMut(&str, &str) -> String) -> io::Result<()> {
    let lang_root = root.join("assets/resources/lang");
    let english: Map<String, Value> = serde_json::from_str(&fs::read_to_string(lang_root.join("en_us.json"))?)?;
    let output: Map<String, Value> = english
        .iter()
        .map(|(key, value)| {
            let text = if key == "language" { language.to_owned() } else { transform(key, value.as_str().unwrap_or_default()) };
            (key.clone(), Value::String(text))
        })
        .collect();
    write_json(lang_root.join(file), &Value::Object(output))
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '\'' || c == '-'
}

/// Splits text into runs of word characters and single other characters.
fn tokens(text: &str) -> impl Iterator<Item = &str> {
    let mut rest = text;
    std::iter::from_fn(move || {
        let first = rest.chars().next()?;
        let end = if is_word_char(first) {
            rest.find(|c| !is_word_char(c)).unwrap_or(rest.len())
        } else {
            first.len_utf8()
        };
        let (token, tail) = rest.split_at(end);
        rest = tail;
        Some(token)
    })
}

/// Small deterministic generator (SplitMix64) so output is stable across runs.
struct Rng(u64);
impl Rng {
    fn seeded(key: &str) -> Self {
        // FNV-1a
        Self(key.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)))
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = usize::try_from(self.next() % (i as u64 + 1)).unwrap_or(0);
            items.swap(i, j);
        }
    }
}

/// Writes JSON with four-space indentation and non-ASCII escaped, matching the
/// existing generated files.
fn write_json(path: PathBuf, value: &Value) -> io::Result<()> {
    let mut out = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut out, AsciiFormatter(serde_json::ser::PrettyFormatter::with_indent(b"    ")));
    serde::Serialize::serialize(value, &mut serializer)?;
    fs::write(path, out)
}

struct AsciiFormatter<'a>(serde_json::ser::PrettyFormatter<'a>);
impl serde_json::ser::Formatter for AsciiFormatter<'_> {
    fn write_string_fragment<W: ?Sized + io::Write>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()> {
        for c in fragment.chars() {
            if c.is_ascii() {
                write!(writer, "{c}")?;
            } else {
                for unit in c.encode_utf16(&mut [0; 2]) {
                    write!(writer, "\\u{unit:04x}")?;
                }
            }
        }
        Ok(())
    }
    fn begin_array<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> { self.0.begin_array(w) }
    fn end_array<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> { self.0.end_array(w) }
    fn begin_array_value<W: ?Sized + io::Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> { self.0.begin_array_value(w, first) }
    fn end_array_value<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> { self.0.end_array_value(w) }
    fn begin_object<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> { self.0.begin_object(w) }
    fn end_object<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> { self.0.end_object(w) }
    fn begin_object_key<W: ?Sized + io::Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> { self.0.begin_object_key(w, first) }
    fn begin_object_value<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> { self.0.begin_object_value(w) }
    fn end_object_value<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> { self.0.end_object_value(w) }
}
