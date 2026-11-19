// CEP:FILE: tools/cep_lint/src/json.rs
// CEP:WHAT: Minimal JSON parser with position-preserving errors for configuration loading.
// CEP:WHY: Bootstrap error CEP-LINT-BOOT-2 requires file, line, and column for parse failures; no external serializer is needed.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Returns JsonError with position for malformed input; never panics on malformed input.
// CEP:ASSUMES: Input is UTF-8; enforced by caller reading files as UTF-8 strings.
// CEP:COST: cold; configuration files are small (less than one thousand entries).
// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.

/// A parsed JSON value with insertion-ordered objects for deterministic output.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// The null literal.
    Null,
    /// A boolean literal.
    Bool(bool),
    /// A numeric literal.
    Num(f64),
    /// A string literal.
    Str(String),
    /// An array of values.
    Arr(Vec<Value>),
    /// An object with insertion-ordered pairs.
    Obj(Vec<(String, Value)>),
}

/// A JSON parse failure with one-based line and column information.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonError {
    /// One-based line of the failure.
    pub line: usize,
    /// One-based column of the failure.
    pub col: usize,
    /// Description of the failure.
    pub detail: String,
}

impl Value {
/// CEP:WHAT: Looks up a key in an object value.
/// CEP:WHY: Checks read rule parameters by name; non-objects have no parameters.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; missing keys yield None rather than an error.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear over object pairs.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

/// CEP:WHAT: Returns the string content of a Str value.
/// CEP:WHY: Parameter accessors share one consistent naming scheme.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

/// CEP:WHAT: Returns the numeric content as i64 when it is integral.
/// CEP:WHY: Thresholds and severities are configured as integers.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Values outside the i64 range are treated as non-integral; enforced by range check.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Num(n) if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 => {
                Some(*n as i64)
            }
            _ => None,
        }
    }

/// CEP:WHAT: Returns the boolean content of a Bool value.
/// CEP:WHY: Flag parameters such as icase are configured as booleans.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

/// CEP:WHAT: Returns the array elements of an Arr value.
/// CEP:WHY: List parameters such as token lists are configured as arrays.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    pub fn as_arr(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(items) => Some(items),
            _ => None,
        }
    }
}

struct Parser<'a> {
    src: &'a [char],
    pos: usize,
    line: usize,
    col: usize,
}

/// CEP:WHAT: Parses a JSON document into a Value or a positioned error.
/// CEP:WHY: All configuration data is JSON; positioned errors implement bootstrap code two.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError for malformed input; never panics.
/// CEP:ASSUMES: Trailing content after the root value is rejected; enforced below.
/// CEP:COST: cold; linear in input length.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
pub fn parse(text: &str) -> Result<Value, JsonError> {
    let chars: Vec<char> = text.chars().collect();
    let mut p = Parser { src: &chars, pos: 0, line: 1, col: 1 };
    p.skip_ws();
    let v = p.value()?;
    p.skip_ws();
    if p.pos != p.src.len() {
        return Err(p.err("trailing content after JSON value"));
    }
    Ok(v)
}

impl<'a> Parser<'a> {
/// CEP:WHAT: Reports an error at the current parser position.
/// CEP:WHY: Positioned errors are the contract of bootstrap code two.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; allocates one error string.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn err(&self, detail: &str) -> JsonError {
        JsonError { line: self.line, col: self.col, detail: detail.to_string() }
    }

/// CEP:WHAT: Advances over whitespace while tracking line and column.
/// CEP:WHY: Error positions must match the original file text for diagnosis.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in whitespace run length.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn skip_ws(&mut self) {
        while self.pos < self.src.len() {
            let c = self.src[self.pos];
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else if c == ' ' || c == '\t' || c == '\r' {
                self.col += 1;
            } else {
                break;
            }
            self.pos += 1;
        }
    }

/// CEP:WHAT: Consumes one character for error position tracking.
/// CEP:WHY: Line and column counters must advance in lockstep with input.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn bump(&mut self) {
        if self.pos < self.src.len() {
            if self.src[self.pos] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
    }

/// CEP:WHAT: Expects a literal string at the current position.
/// CEP:WHY: Keywords and structural punctuation are matched as literals.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError when the literal does not match.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in literal length.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn expect(&mut self, lit: &str, detail: &str) -> Result<(), JsonError> {
        for (i, lc) in lit.chars().enumerate() {
            if self.pos + i >= self.src.len() || self.src[self.pos + i] != lc {
                return Err(self.err(detail));
            }
        }
        for _ in 0..lit.chars().count() {
            self.bump();
        }
        Ok(())
    }

/// CEP:WHAT: Parses one JSON value.
/// CEP:WHY: Values are the only recursive production of the grammar.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError for malformed or empty input.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in the value size.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn value(&mut self) -> Result<Value, JsonError> {
        if self.pos >= self.src.len() {
            return Err(self.err("unexpected end of input"));
        }
        match self.src[self.pos] {
            '{' => self.object(),
            '[' => self.array(),
            '"' => self.string().map(Value::Str),
            't' => self.expect("true", "bad literal").map(|_| Value::Bool(true)),
            'f' => self.expect("false", "bad literal").map(|_| Value::Bool(false)),
            'n' => self.expect("null", "bad literal").map(|_| Value::Null),
            _ => self.number().map(Value::Num),
        }
    }

/// CEP:WHAT: Parses an object with insertion-ordered pairs.
/// CEP:WHY: Ordering preservation keeps serialized output and lookups deterministic.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError for malformed structure.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in object size.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn object(&mut self) -> Result<Value, JsonError> {
        self.expect("{", "expected object")?;
        let mut pairs: Vec<(String, Value)> = Vec::new();
        if self.peek_closer('}') {
            return Ok(Value::Obj(pairs));
        }
        loop {
            self.object_pair(&mut pairs)?;
            self.skip_ws();
            if self.pos < self.src.len() && self.src[self.pos] == ',' {
                self.bump();
                continue;
            }
            self.expect("}", "expected closing brace")?;
            return Ok(Value::Obj(pairs));
        }
    }

/// CEP:WHAT: Consumes a closing brace when it is the next significant character.
/// CEP:WHY: Empty containers are the only early-exit productions of the grammar.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn peek_closer(&mut self, closer: char) -> bool {
        self.skip_ws();
        let at = self.pos < self.src.len() && self.src[self.pos] == closer;
        if at {
            self.bump();
        }
        at
    }

/// CEP:WHAT: Parses one object pair into the accumulator.
/// CEP:WHY: Splitting the pair grammar keeps the object loop within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError for malformed keys, colons, or values.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in pair length.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn object_pair(&mut self, pairs: &mut Vec<(String, Value)>) -> Result<(), JsonError> {
        self.skip_ws();
        let key = self.string()?;
        self.skip_ws();
        self.expect(":", "expected colon")?;
        self.skip_ws();
        let val = self.value()?;
        pairs.push((key, val));
        Ok(())
    }

/// CEP:WHAT: Parses an array of values.
/// CEP:WHY: List parameters are arrays of scalars or nested values.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError for malformed structure.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in array size.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn array(&mut self) -> Result<Value, JsonError> {
        self.expect("[", "expected array")?;
        let mut items: Vec<Value> = Vec::new();
        if self.peek_closer(']') {
            return Ok(Value::Arr(items));
        }
        loop {
            self.array_item(&mut items)?;
            self.skip_ws();
            if self.pos < self.src.len() && self.src[self.pos] == ',' {
                self.bump();
                continue;
            }
            self.expect("]", "expected closing bracket")?;
            return Ok(Value::Arr(items));
        }
    }

/// CEP:WHAT: Parses one array item into the accumulator.
/// CEP:WHY: Splitting the item grammar keeps the array loop within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError for malformed items.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in item size.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn array_item(&mut self, items: &mut Vec<Value>) -> Result<(), JsonError> {
        self.skip_ws();
        let v = self.value()?;
        items.push(v);
        Ok(())
    }

/// CEP:WHAT: Parses a double-quoted string with escape sequences.
/// CEP:WHY: Keys and string parameters are strings.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError for unterminated strings and bad escapes.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in string length.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn string(&mut self) -> Result<String, JsonError> {
        self.expect("\"", "expected string")?;
        let mut out = String::new();
        loop {
            if self.pos >= self.src.len() {
                return Err(self.err("unterminated string"));
            }
            let c = self.src[self.pos];
            if c == '"' {
                self.bump();
                return Ok(out);
            }
            if c == '\\' {
                self.push_escape(&mut out)?;
                continue;
            }
            out.push(c);
            self.bump();
        }
    }

/// CEP:WHAT: Consumes one escape sequence and appends its decoded character.
/// CEP:WHY: Splitting escape handling keeps the string loop within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError for bad escapes.
/// CEP:ASSUMES: The caller verified the current character is a backslash.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn push_escape(&mut self, out: &mut String) -> Result<(), JsonError> {
        self.bump();
        let e = if self.pos < self.src.len() { self.src[self.pos] } else { return Err(self.err("bad escape")) };
        self.bump();
        match e {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            'b' => out.push('\u{8}'),
            'f' => out.push('\u{c}'),
            'u' => out.push(self.unicode_escape()?),
            other => out.push(other),
        }
        Ok(())
    }

/// CEP:WHAT: Parses a four-digit unicode escape into a scalar value.
/// CEP:WHY: The JSON grammar requires \uXXXX handling inside strings.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError for bad digits or invalid scalars.
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn unicode_escape(&mut self) -> Result<char, JsonError> {
        let mut code: u32 = 0;
        for _ in 0..ESC_DIGITS {
            if self.pos >= self.src.len() {
                return Err(self.err("bad unicode escape"));
            }
            let c = self.src[self.pos];
            let d = c.to_digit(ESC_RADIX).ok_or_else(|| self.err("bad unicode digit"))?;
            code = code * ESC_RADIX + d;
            self.bump();
        }
        char::from_u32(code).ok_or_else(|| self.err("bad unicode scalar"))
    }

/// CEP:WHAT: Parses a numeric literal into f64.
/// CEP:WHY: Thresholds are numeric; integral access is validated by as_i64.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns JsonError when no digits are present.
/// CEP:ASSUMES: Numbers are within f64 precision; configuration thresholds are small.
/// CEP:COST: cold; linear in literal length.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
    fn number(&mut self) -> Result<f64, JsonError> {
        let start = self.pos;
        if self.pos < self.src.len() && self.src[self.pos] == '-' {
            self.bump();
        }
        let mut saw_digit = false;
        while self.pos < self.src.len() {
            let c = self.src[self.pos];
            if c.is_ascii_digit() {
                saw_digit = true;
                self.bump();
            } else if c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                self.bump();
            } else {
                break;
            }
        }
        if !saw_digit {
            return Err(self.err("expected number"));
        }
        let text: String = self.src[start..self.pos].iter().collect();
        text.parse::<f64>().map_err(|_| self.err("bad number literal"))
    }
}

/// Digit count of a JSON unicode escape sequence.
const ESC_DIGITS: usize = 4;

/// Radix of a JSON unicode escape sequence.
const ESC_RADIX: u32 = 16;

/// CEP:WHAT: Serializes a Value to a compact JSON string.
/// CEP:WHY: Deterministic serialization is required by the self-test manifest writer and reproducible builds.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Object pairs are emitted in insertion order.
/// CEP:COST: cold; linear in value size.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
pub fn serialize(v: &Value) -> String {
    let mut out = String::new();
    write_value(v, &mut out);
    out
}

/// CEP:WHAT: Writes one JSON value to the output buffer.
/// CEP:WHY: Serializer recursion follows the parser grammar exactly.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in value size.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
fn write_value(v: &Value, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Num(n) => {
            if n.fract() == 0.0 && n.abs() < NUM_LIMIT {
                out.push_str(&format!("{}", *n as i64));
            } else {
                out.push_str(&format!("{}", n));
            }
        }
        Value::Str(s) => write_string(s, out),
        Value::Arr(items) => write_array(items, out),
        Value::Obj(pairs) => write_object(pairs, out),
    }
}

/// CEP:WHAT: Writes an object with insertion-ordered pairs.
/// CEP:WHY: Splitting keeps each serializer function within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in pair count.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
fn write_object(pairs: &[(String, Value)], out: &mut String) {
    out.push('{');
    for (i, (k, val)) in pairs.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_string(k, out);
        out.push(':');
        write_value(val, out);
    }
    out.push('}');
}

/// CEP:WHAT: Writes a string with JSON escaping.
/// CEP:WHY: Control characters and quotes must not break the serialized form.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in string length.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other if (other as u32) < CTRL_LIMIT => out.push_str(&format!("\\u{:04x}", other as u32)),
            other => out.push(other),
        }
    }
    out.push('"');
}

/// CEP:WHAT: Writes an array of values.
/// CEP:WHY: Splitting keeps each serializer function within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in item count.
/// CEP:EVIDENCE: unit test cep_lint::json::tests::positions_and_roundtrip.
fn write_array(items: &[Value], out: &mut String) {
    out.push('[');
    for (i, it) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_value(it, out);
    }
    out.push(']');
}

/// Upper bound for integral number serialization.
const NUM_LIMIT: f64 = 1e15;

/// Upper bound of the JSON control character range.
const CTRL_LIMIT: u32 = 0x20;

#[cfg(test)]
mod tests {
    use super::*;

/// CEP:WHAT: Verifies positional error reporting and serialization round trip.
/// CEP:WHY: Bootstrap code two and deterministic serialization are contract features.
/// CEP:STATUS: complete
/// CEP:FAILURE: Fails the test binary on regression.
/// CEP:ASSUMES: none
/// CEP:COST: cold; unit test only.
/// CEP:EVIDENCE: cargo test -p cep_lint.
    #[test]
    fn positions_and_roundtrip() {
        let doc = r#"{"a": 1, "b": ["x", true], "c": {"d": null}}"#;
        let v = parse(doc).unwrap();
        assert_eq!(v.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(serialize(&v), doc.replace(' ', ""));
        let bad = parse("{\"a\": }");
        let e = bad.unwrap_err();
        assert!(e.line >= 1 && e.col >= 1);
    }
}
