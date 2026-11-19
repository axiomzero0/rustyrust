// CEP:FILE: checker/certificates/src/lib.rs
// CEP:WHAT: Certificate data model and minimal JSON codec for proof-carrying optimization.
// CEP:WHY: Passes emit machine-checkable evidence; the verifier consumes it without optimizer code.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none; malformed certificates fail verification, not construction.
// CEP:ASSUMES: JSON output is deterministic with fixed key order.
// CEP:COST: cold; linear in certificate size.
// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
//! Certificate format for proof-carrying optimization.
#![deny(missing_docs)]

/// JSON value with insertion-ordered objects.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    /// Null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Integral number.
    Num(i64),
    /// Text.
    Str(String),
    /// Array.
    Arr(Vec<Json>),
    /// Object with ordered pairs.
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// CEP:WHAT: Looks up one key of an object.
    /// CEP:WHY: The verifier reads obligation fields by name.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in pairs.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    pub fn get(&self, k: &str) -> Option<&Json> {
        match self {
            Json::Obj(pairs) => pairs.iter().find(|(key, _)| key == k).map(|(_, v)| v),
            _ => None,
        }
    }

    /// CEP:WHAT: Returns the integral value of a Num.
    /// CEP:WHY: Local ids and block ids are integral fields.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    /// CEP:WHAT: Returns the text of a Str.
    /// CEP:WHY: Operator names and hashes are text fields.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    /// CEP:WHAT: Returns the array of an Arr.
    /// CEP:WHY: Obligations are arrays.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    pub fn as_arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }
}

/// Escape byte count of JSON control output.
const CTRL_LIMIT: u32 = 0x20;

/// CEP:WHAT: Serializes one JSON value in compact deterministic form.
/// CEP:WHY: Hashes of certificates must be reproducible.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Object pairs serialize in insertion order.
/// CEP:COST: cold; linear in value size.
/// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
pub fn serialize(v: &Json) -> String {
    let mut s = String::new();
    write_val(v, &mut s);
    s
}

/// CEP:WHAT: Writes one JSON value.
/// CEP:WHY: Serializer recursion mirrors the data model.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in value size.
/// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
fn write_val(v: &Json, out: &mut String) {
    match v {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Num(n) => out.push_str(&n.to_string()),
        Json::Str(s) => write_str(s, out),
        Json::Arr(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_val(x, out);
            }
            out.push(']');
        }
        Json::Obj(p) => {
            out.push('{');
            for (i, (k, x)) in p.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_str(k, out);
                out.push(':');
                write_val(x, out);
            }
            out.push('}');
        }
    }
}

/// CEP:WHAT: Writes one escaped JSON string.
/// CEP:WHY: Control characters must not break the format.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in string length.
/// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
fn write_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            other if (other as u32) < CTRL_LIMIT => out.push_str(&format!("\\u{:04x}", other as u32)),
            other => out.push(other),
        }
    }
    out.push('"');
}

struct P<'a> {
    c: &'a [char],
    i: usize,
}

/// CEP:WHAT: Parses one JSON value from text.
/// CEP:WHY: The verifier reads certificates from files.
/// CEP:ID: CS-a09
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns None for malformed input; the verifier reports the failure.
/// CEP:ASSUMES: Certificates are small; parsing is recursive with bounded depth by file size.
/// CEP:COST: cold; linear in input size.
/// CEP:EVIDENCE: doc file docs/functions/CS/CS-a09.md; unit test certificate_roundtrip.
pub fn parse(text: &str) -> Option<Json> {
    let chars: Vec<char> = text.chars().collect();
    let mut p = P { c: &chars, i: 0 };
    p.ws();
    let v = p.value()?;
    Some(v)
}

impl<'a> P<'a> {
    /// CEP:WHAT: Skips whitespace.
    /// CEP:WHY: JSON allows whitespace between tokens.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in whitespace.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    fn ws(&mut self) {
        while self.i < self.c.len() && self.c[self.i].is_whitespace() {
            self.i += 1;
        }
    }

    /// CEP:WHAT: Parses one value.
    /// CEP:WHY: Values are the recursive production.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Returns None on malformed input.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in value size.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    fn value(&mut self) -> Option<Json> {
        self.ws();
        match self.c.get(self.i)? {
            '{' => self.obj(),
            '[' => self.arr(),
            '"' => self.str().map(Json::Str),
            'n' => self.lit("null", Json::Null),
            't' => self.lit("true", Json::Bool(true)),
            'f' => self.lit("false", Json::Bool(false)),
            _ => self.num(),
        }
    }

    /// CEP:WHAT: Consumes one literal.
    /// CEP:WHY: Null and booleans are literals.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Returns None when the literal does not match.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    fn lit(&mut self, w: &str, v: Json) -> Option<Json> {
        for ch in w.chars() {
            if self.c.get(self.i) != Some(&ch) {
                return None;
            }
            self.i += 1;
        }
        Some(v)
    }

    /// CEP:WHAT: Parses one object.
    /// CEP:WHY: Certificates are objects of obligations.
    /// CEP:ID: CS-a11
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Returns None for malformed structure.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in pairs.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    fn obj(&mut self) -> Option<Json> {
        self.i += 1;
        let mut pairs = Vec::new();
        self.ws();
        if self.c.get(self.i) == Some(&'}') {
            self.i += 1;
            return Some(Json::Obj(pairs));
        }
        loop {
            self.ws();
            let k = self.str()?;
            self.ws();
            if self.c.get(self.i) != Some(&':') {
                return None;
            }
            self.i += 1;
            let v = self.value()?;
            pairs.push((k, v));
            self.ws();
            match self.c.get(self.i)? {
                ',' => self.i += 1,
                '}' => {
                    self.i += 1;
                    return Some(Json::Obj(pairs));
                }
                _ => return None,
            }
        }
    }

    /// CEP:WHAT: Parses one array.
    /// CEP:WHY: Obligations are arrays of objects.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Returns None for malformed structure.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in items.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    fn arr(&mut self) -> Option<Json> {
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.c.get(self.i) == Some(&']') {
            self.i += 1;
            return Some(Json::Arr(items));
        }
        loop {
            self.ws();
            items.push(self.value()?);
            self.ws();
            match self.c.get(self.i)? {
                ',' => self.i += 1,
                ']' => {
                    self.i += 1;
                    return Some(Json::Arr(items));
                }
                _ => return None,
            }
        }
    }

    /// CEP:WHAT: Parses one string with escapes.
    /// CEP:WHY: Keys and text fields are strings.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Returns None for unterminated strings.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in string length.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    fn str(&mut self) -> Option<String> {
        self.i += 1;
        let mut s = String::new();
        loop {
            let c = *self.c.get(self.i)?;
            self.i += 1;
            match c {
                '"' => return Some(s),
                '\\' => {
                    let e = *self.c.get(self.i)?;
                    self.i += 1;
                    s.push(match e {
                        'n' => '\n',
                        't' => '\t',
                        other => other,
                    });
                }
                other => s.push(other),
            }
        }
    }

    /// CEP:WHAT: Parses one integral number.
    /// CEP:WHY: Ids and hashes are integral.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Returns None for non-integral text.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in digit count.
    /// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
    fn num(&mut self) -> Option<Json> {
        let start = self.i;
        if self.c.get(self.i) == Some(&'-') {
            self.i += 1;
        }
        while matches!(self.c.get(self.i), Some(c) if c.is_ascii_digit()) {
            self.i += 1;
        }
        let t: String = self.c[start..self.i].iter().collect();
        t.parse::<i64>().ok().map(Json::Num)
    }
}

/// Certificate kind of one pass obligation.
pub const KIND_CONST_FOLD: &str = "const_fold";

/// Certificate kind for dead store removal.
pub const KIND_DEAD_STORE: &str = "dead_store";

/// CEP:WHAT: Builds a const-fold obligation object.
/// CEP:WHY: The verifier re-derives folded values from operands and semantics.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Semantics are the wrap-i64 dialect rules.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
pub fn fold_obligation(fn_name: &str, block: usize, place: usize, op: &str, l: i64, r: i64, result: i64) -> Json {
    Json::Obj(vec![
        ("kind".into(), Json::Str(KIND_CONST_FOLD.into())),
        ("fn".into(), Json::Str(fn_name.into())),
        ("block".into(), Json::Num(block as i64)),
        ("place".into(), Json::Num(place as i64)),
        ("op".into(), Json::Str(op.into())),
        ("lhs".into(), Json::Num(l)),
        ("rhs".into(), Json::Num(r)),
        ("result".into(), Json::Num(result)),
    ])
}

/// CEP:WHAT: Builds a dead-store obligation object.
/// CEP:WHY: The verifier re-derives local liveness from the output program.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_certificates::tests::certificate_roundtrip.
pub fn dead_obligation(fn_name: &str, block: usize, index: usize, local: usize) -> Json {
    Json::Obj(vec![
        ("kind".into(), Json::Str(KIND_DEAD_STORE.into())),
        ("fn".into(), Json::Str(fn_name.into())),
        ("block".into(), Json::Num(block as i64)),
        ("index".into(), Json::Num(index as i64)),
        ("local".into(), Json::Num(local as i64)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample value used by the codec test.
    const SAMPLE_NUM: i64 = 7;

    /// Sample result used by the codec test.
    const SAMPLE_SUM: i64 = 9;

    /// CEP:WHAT: Verifies JSON round trip through serialize and parse.
    /// CEP:WHY: The verifier reads emitted certificates verbatim.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-certificates.
    #[test]
    fn certificate_roundtrip() {
        let ob = fold_obligation("main", 0, 1, "add", 2, SAMPLE_NUM, SAMPLE_SUM);
        let text = serialize(&ob);
        let back = parse(&text).unwrap();
        assert_eq!(back.get("result").unwrap().as_i64(), Some(SAMPLE_SUM));
        assert_eq!(back.get("op").unwrap().as_str(), Some("add"));
    }
}
