//! Reading one fragment file into a tree of keys and leaves.
//!
//! Every format is read into the same [`Node`] tree, so the rest of the crate
//! never sees a parser type. Three rules hold in every format: a key set twice
//! is refused, the tree nests at most [`MAX_DEPTH`] levels, and values stay as
//! written (nothing is resolved or converted).

use std::collections::HashSet;

use toml_edit::{DocumentMut, Item, Table, Value};
use yaml_rust2::parser::{Event, Parser};
use yaml_rust2::scanner::TScalarStyle;

use crate::display::{display_name, toml_reason};
use crate::manifest::split_path;
use crate::source::FragmentFormat;

/// The most levels of objects, tables, mappings and arrays one fragment holds.
pub(crate) const MAX_DEPTH: usize = 64;

/// One entry of a fragment: an object, or a leaf. An array is a leaf. A leaf
/// holds its text when the value is a string, and nothing for any other
/// value.
#[derive(Debug, Clone)]
pub(crate) enum Node {
    Object(Vec<(String, Node)>),
    Leaf(Option<String>),
}

impl Node {
    fn child(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, n)| n),
            Node::Leaf(_) => None,
        }
    }

    /// The entry at `path`, a leaf or an object.
    pub(crate) fn at(&self, path: &[String]) -> Option<&Node> {
        let mut node = self;
        for segment in path {
            node = node.child(segment)?;
        }
        Some(node)
    }

    /// Every leaf path under this node, in the order written.
    pub(crate) fn leaf_paths(&self) -> Vec<Vec<String>> {
        let mut out = Vec::new();
        let mut prefix = Vec::new();
        self.collect(&mut prefix, &mut out);
        out
    }

    fn collect(&self, prefix: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
        match self {
            Node::Object(entries) => {
                for (key, node) in entries {
                    prefix.push(key.clone());
                    node.collect(prefix, out);
                    prefix.pop();
                }
            }
            Node::Leaf(_) => out.push(prefix.clone()),
        }
    }
}

/// A key path in the printed form: segments joined by dots, a segment that
/// holds a dot, a double quote or whitespace written as a JSON string.
pub(crate) fn print_path(segments: &[String]) -> String {
    let parts: Vec<String> = segments
        .iter()
        .map(|segment| {
            let needs_quote = segment
                .chars()
                .any(|c| c == '.' || c == '"' || c.is_whitespace());
            if needs_quote {
                serde_json::to_string(segment).unwrap_or_else(|_| segment.clone())
            } else {
                segment.clone()
            }
        })
        .collect();
    parts.join(".")
}

/// The segments of a printed key path, or `None` when a quoted segment does
/// not decode.
pub(crate) fn parse_path(path: &str) -> Option<Vec<String>> {
    let mut segments = Vec::new();
    for segment in split_path(path) {
        if segment.starts_with('"') {
            segments.push(serde_json::from_str::<String>(segment).ok()?);
        } else {
            segments.push(segment.to_string());
        }
    }
    Some(segments)
}

/// Reads `text` in `format`. The top level is an object in every format. The
/// error text can hold a key from the file, so it is shown through
/// [`display_name`] by the caller.
pub(crate) fn parse(format: FragmentFormat, text: &str) -> Result<Node, String> {
    let node = match format {
        FragmentFormat::Json => parse_json(text)?,
        FragmentFormat::Toml => parse_toml(text)?,
        FragmentFormat::Yaml => parse_yaml(text)?,
    };
    match node {
        Node::Object(_) => Ok(node),
        Node::Leaf(_) => Err("the top level is not a mapping of keys".to_string()),
    }
}

fn twice(segments: &[String], key: &str) -> String {
    let mut path = segments.to_vec();
    path.push(key.to_string());
    format!("the key `{}` is set twice", print_path(&path))
}

fn too_deep() -> String {
    format!("the file nests deeper than {MAX_DEPTH} levels")
}

// JSON: a reader of its own, because a value tree would keep the last of a
// repeated key and lose the order the keys were written in.

fn parse_json(text: &str) -> Result<Node, String> {
    let mut reader = Json {
        text,
        bytes: text.as_bytes(),
        pos: 0,
        path: Vec::new(),
    };
    let node = reader.value(0)?;
    reader.skip_space();
    if reader.pos != reader.bytes.len() {
        return Err("not valid JSON: text after the value".to_string());
    }
    Ok(node)
}

struct Json<'a> {
    text: &'a str,
    bytes: &'a [u8],
    pos: usize,
    path: Vec<String>,
}

impl Json<'_> {
    fn skip_space(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn string(&mut self) -> Result<String, String> {
        let start = self.pos;
        self.pos += 1;
        loop {
            match self.peek() {
                None => return Err("not valid JSON: a string never ends".to_string()),
                Some(b'\\') => self.pos += 2,
                Some(b'"') => {
                    self.pos += 1;
                    break;
                }
                Some(_) => self.pos += 1,
            }
        }
        let raw = self
            .text
            .get(start..self.pos)
            .ok_or_else(|| "not valid JSON: a string never ends".to_string())?;
        serde_json::from_str::<String>(raw).map_err(|err| format!("not valid JSON: {err}"))
    }

    fn value(&mut self, depth: usize) -> Result<Node, String> {
        self.skip_space();
        match self.peek() {
            None => Err("not valid JSON: the text ends early".to_string()),
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => Ok(Node::Leaf(Some(self.string()?))),
            Some(_) => self.token(),
        }
    }

    fn token(&mut self) -> Result<Node, String> {
        let start = self.pos;
        while let Some(byte) = self.peek() {
            if matches!(byte, b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r') {
                break;
            }
            self.pos += 1;
        }
        let token = self.text.get(start..self.pos).unwrap_or("");
        let plain = matches!(token.as_bytes().first(), Some(b'-' | b'0'..=b'9'))
            || matches!(token, "true" | "false" | "null");
        if plain && serde_json::from_str::<serde_json::Value>(token).is_ok() {
            Ok(Node::Leaf(None))
        } else {
            Err("not valid JSON: an unexpected value".to_string())
        }
    }

    fn object(&mut self, depth: usize) -> Result<Node, String> {
        if depth >= MAX_DEPTH {
            return Err(too_deep());
        }
        self.pos += 1;
        let mut entries: Vec<(String, Node)> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        self.skip_space();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Node::Object(entries));
        }
        loop {
            self.skip_space();
            if self.peek() != Some(b'"') {
                return Err("not valid JSON: a key is not a string".to_string());
            }
            let key = self.string()?;
            if !seen.insert(key.clone()) {
                return Err(twice(&self.path, &key));
            }
            self.skip_space();
            if self.peek() != Some(b':') {
                return Err("not valid JSON: a key has no `:`".to_string());
            }
            self.pos += 1;
            self.path.push(key.clone());
            let node = self.value(depth + 1)?;
            self.path.pop();
            entries.push((key, node));
            self.skip_space();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Node::Object(entries));
                }
                _ => return Err("not valid JSON: an object never closes".to_string()),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Node, String> {
        if depth >= MAX_DEPTH {
            return Err(too_deep());
        }
        self.pos += 1;
        self.skip_space();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Node::Leaf(None));
        }
        let mut index = 0usize;
        loop {
            self.path.push(format!("[{index}]"));
            self.value(depth + 1)?;
            self.path.pop();
            index += 1;
            self.skip_space();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Node::Leaf(None));
                }
                _ => return Err("not valid JSON: an array never closes".to_string()),
            }
        }
    }
}

// TOML: toml_edit refuses a repeated key itself.

fn parse_toml(text: &str) -> Result<Node, String> {
    let doc: DocumentMut = text.parse().map_err(|err: toml_edit::TomlError| {
        format!("not valid TOML: {}", toml_reason(&err, text))
    })?;
    toml_table(doc.as_table(), 1)
}

fn toml_table(table: &Table, depth: usize) -> Result<Node, String> {
    if depth > MAX_DEPTH {
        return Err(too_deep());
    }
    let mut entries = Vec::new();
    for (key, item) in table {
        entries.push((key.to_string(), toml_item(item, depth + 1)?));
    }
    Ok(Node::Object(entries))
}

fn toml_item(item: &Item, depth: usize) -> Result<Node, String> {
    match item {
        Item::Table(table) => toml_table(table, depth),
        Item::Value(value) => toml_value(value, depth),
        Item::ArrayOfTables(_) | Item::None => Ok(Node::Leaf(None)),
    }
}

fn toml_value(value: &Value, depth: usize) -> Result<Node, String> {
    match value {
        Value::String(text) => Ok(Node::Leaf(Some(text.value().clone()))),
        Value::InlineTable(table) => {
            if depth > MAX_DEPTH {
                return Err(too_deep());
            }
            let mut entries = Vec::new();
            for (key, inner) in table {
                entries.push((key.to_string(), toml_value(inner, depth + 1)?));
            }
            Ok(Node::Object(entries))
        }
        _ => Ok(Node::Leaf(None)),
    }
}

// YAML: one document, no anchors, aliases or tags, string keys. The parser's
// own event stream is read one event at a time, so a deep file stops at the
// cap before anything recurses.

enum Frame {
    Map {
        entries: Vec<(String, Node)>,
        key: Option<String>,
        seen: HashSet<String>,
    },
    Seq,
}

fn parse_yaml(text: &str) -> Result<Node, String> {
    let mut parser = Parser::new_from_str(text);
    let mut stack: Vec<Frame> = Vec::new();
    let mut root: Option<Node> = None;
    let mut documents = 0usize;
    loop {
        let (event, _) = parser
            .next_token()
            .map_err(|err| format!("not valid YAML: {}", err.info()))?;
        match event {
            Event::Nothing | Event::StreamStart | Event::DocumentEnd => {}
            Event::StreamEnd => break,
            Event::DocumentStart => {
                documents += 1;
                if documents > 1 {
                    return Err("the file holds more than one YAML document".to_string());
                }
            }
            Event::Alias(_) => return Err("the file uses a YAML alias".to_string()),
            Event::Scalar(value, style, anchor, tag) => {
                if anchor != 0 {
                    return Err("the file uses a YAML anchor".to_string());
                }
                if tag.is_some() {
                    return Err("the file uses a YAML tag".to_string());
                }
                let is_string = style != TScalarStyle::Plain || !plain_non_string(&value);
                if stack.is_empty() {
                    if value.is_empty() || matches!(value.as_str(), "~" | "null" | "Null" | "NULL")
                    {
                        root = Some(Node::Object(Vec::new()));
                        continue;
                    }
                    return Err("the top level is not a mapping of keys".to_string());
                }
                put_scalar(&mut stack, &mut root, value, is_string)?;
            }
            Event::SequenceStart(anchor, tag) => {
                no_anchor_or_tag(anchor, tag.is_some())?;
                open(&mut stack, Frame::Seq)?;
            }
            Event::MappingStart(anchor, tag) => {
                no_anchor_or_tag(anchor, tag.is_some())?;
                open(
                    &mut stack,
                    Frame::Map {
                        entries: Vec::new(),
                        key: None,
                        seen: HashSet::new(),
                    },
                )?;
            }
            Event::SequenceEnd | Event::MappingEnd => {
                let node = match stack.pop() {
                    Some(Frame::Map { entries, .. }) => Node::Object(entries),
                    Some(Frame::Seq) => Node::Leaf(None),
                    None => return Err("not valid YAML: a collection ends twice".to_string()),
                };
                finish(&mut stack, &mut root, node);
            }
        }
    }
    Ok(root.unwrap_or(Node::Object(Vec::new())))
}

fn no_anchor_or_tag(anchor: usize, has_tag: bool) -> Result<(), String> {
    if anchor != 0 {
        return Err("the file uses a YAML anchor".to_string());
    }
    if has_tag {
        return Err("the file uses a YAML tag".to_string());
    }
    Ok(())
}

fn open(stack: &mut Vec<Frame>, frame: Frame) -> Result<(), String> {
    if stack.len() >= MAX_DEPTH {
        return Err(too_deep());
    }
    if matches!(stack.last(), Some(Frame::Map { key: None, .. })) {
        return Err("a YAML key is not a string".to_string());
    }
    stack.push(frame);
    Ok(())
}

fn put_scalar(
    stack: &mut [Frame],
    root: &mut Option<Node>,
    value: String,
    is_string: bool,
) -> Result<(), String> {
    if matches!(stack.last(), Some(Frame::Map { key: None, .. })) {
        if !is_string {
            return Err(format!("the YAML key `{value}` is not a string"));
        }
        let repeated = match stack.last_mut() {
            Some(Frame::Map { key, seen, .. }) => {
                let repeated = !seen.insert(value.clone());
                if !repeated {
                    *key = Some(value.clone());
                }
                repeated
            }
            _ => false,
        };
        if repeated {
            let ancestors: Vec<String> = stack
                .iter()
                .filter_map(|frame| match frame {
                    Frame::Map { key: Some(k), .. } => Some(k.clone()),
                    _ => None,
                })
                .collect();
            return Err(twice(&ancestors, &value));
        }
        return Ok(());
    }
    finish(stack, root, Node::Leaf(is_string.then_some(value)));
    Ok(())
}

fn finish(stack: &mut [Frame], root: &mut Option<Node>, node: Node) {
    match stack.last_mut() {
        None => *root = Some(node),
        Some(Frame::Seq) => {}
        Some(Frame::Map { entries, key, .. }) => {
            if let Some(name) = key.take() {
                entries.push((name, node));
            }
        }
    }
}

/// A plain (unquoted) scalar YAML reads as something other than a string:
/// null, a boolean or a number.
fn plain_non_string(text: &str) -> bool {
    if matches!(
        text,
        "" | "~"
            | "null"
            | "Null"
            | "NULL"
            | "true"
            | "True"
            | "TRUE"
            | "false"
            | "False"
            | "FALSE"
    ) {
        return true;
    }
    let unsigned = text.trim_start_matches(['+', '-']);
    if matches!(
        unsigned,
        ".inf" | ".Inf" | ".INF" | ".nan" | ".NaN" | ".NAN"
    ) {
        return true;
    }
    if unsigned.starts_with("0x") || unsigned.starts_with("0o") {
        return unsigned.len() > 2;
    }
    text.bytes().any(|b| b.is_ascii_digit()) && text.parse::<f64>().is_ok()
}

/// Shows a refusal's reason without raw control characters.
pub(crate) fn shown(text: &str) -> String {
    display_name(text)
}
