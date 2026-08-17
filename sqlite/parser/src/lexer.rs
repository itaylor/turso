use crate::{error::Error, token::TokenType, Result};
use turso_macros::match_ignore_ascii_case;

/// Returns true if the given identifier (case-insensitive) is a SQL keyword.
/// This is used to determine whether an identifier needs to be quoted when
/// rendered back to SQL text.
pub fn is_quotable_keyword(input: &[u8]) -> bool {
    let token = keyword_or_id_token(input);
    token != TokenType::TK_ID && token != TokenType::TK_TYPE
}

fn keyword_or_id_token(input: &[u8]) -> TokenType {
    match_ignore_ascii_case!(match input {
        b"ABORT" => TokenType::TK_ABORT,
        b"ACTION" => TokenType::TK_ACTION,
        b"ADD" => TokenType::TK_ADD,
        b"AFTER" => TokenType::TK_AFTER,
        b"ALL" => TokenType::TK_ALL,
        b"ALTER" => TokenType::TK_ALTER,
        b"ALWAYS" => TokenType::TK_ALWAYS,
        b"ANALYZE" => TokenType::TK_ANALYZE,
        b"AND" => TokenType::TK_AND,
        b"AS" => TokenType::TK_AS,
        b"ASC" => TokenType::TK_ASC,
        b"ATTACH" => TokenType::TK_ATTACH,
        b"AUTOINCREMENT" => TokenType::TK_AUTOINCR,
        b"BEFORE" => TokenType::TK_BEFORE,
        b"BEGIN" => TokenType::TK_BEGIN,
        b"BETWEEN" => TokenType::TK_BETWEEN,
        b"BY" => TokenType::TK_BY,
        b"CASCADE" => TokenType::TK_CASCADE,
        b"CASE" => TokenType::TK_CASE,
        b"CAST" => TokenType::TK_CAST,
        b"CHECK" => TokenType::TK_CHECK,
        b"COLLATE" => TokenType::TK_COLLATE,
        b"COLUMN" => TokenType::TK_COLUMNKW,
        b"COMMIT" => TokenType::TK_COMMIT,
        b"CONCURRENT" => TokenType::TK_CONCURRENT,
        b"CONFLICT" => TokenType::TK_CONFLICT,
        b"CONSTRAINT" => TokenType::TK_CONSTRAINT,
        b"CREATE" => TokenType::TK_CREATE,
        b"CROSS" => TokenType::TK_JOIN_KW,
        b"CURRENT" => TokenType::TK_CURRENT,
        b"CURRENT_DATE" => TokenType::TK_CTIME_KW,
        b"CURRENT_TIME" => TokenType::TK_CTIME_KW,
        b"CURRENT_TIMESTAMP" => TokenType::TK_CTIME_KW,
        b"DATABASE" => TokenType::TK_DATABASE,
        b"DEFAULT" => TokenType::TK_DEFAULT,
        b"DEFERRABLE" => TokenType::TK_DEFERRABLE,
        b"DEFERRED" => TokenType::TK_DEFERRED,
        b"DELETE" => TokenType::TK_DELETE,
        b"DESC" => TokenType::TK_DESC,
        b"DETACH" => TokenType::TK_DETACH,
        b"DISTINCT" => TokenType::TK_DISTINCT,
        b"DO" => TokenType::TK_DO,
        b"DROP" => TokenType::TK_DROP,
        b"EACH" => TokenType::TK_EACH,
        b"ELSE" => TokenType::TK_ELSE,
        b"END" => TokenType::TK_END,
        b"ESCAPE" => TokenType::TK_ESCAPE,
        b"EXCEPT" => TokenType::TK_EXCEPT,
        b"EXCLUDE" => TokenType::TK_EXCLUDE,
        b"EXCLUSIVE" => TokenType::TK_EXCLUSIVE,
        b"EXISTS" => TokenType::TK_EXISTS,
        b"EXPLAIN" => TokenType::TK_EXPLAIN,
        b"FAIL" => TokenType::TK_FAIL,
        b"FILTER" => TokenType::TK_FILTER,
        b"FIRST" => TokenType::TK_FIRST,
        b"FOLLOWING" => TokenType::TK_FOLLOWING,
        b"FOR" => TokenType::TK_FOR,
        b"FOREIGN" => TokenType::TK_FOREIGN,
        b"FROM" => TokenType::TK_FROM,
        b"FULL" => TokenType::TK_JOIN_KW,
        b"GENERATED" => TokenType::TK_GENERATED,
        b"GLOB" => TokenType::TK_LIKE_KW,
        b"GROUP" => TokenType::TK_GROUP,
        b"GROUPS" => TokenType::TK_GROUPS,
        b"HAVING" => TokenType::TK_HAVING,
        b"IF" => TokenType::TK_IF,
        b"IGNORE" => TokenType::TK_IGNORE,
        b"IMMEDIATE" => TokenType::TK_IMMEDIATE,
        b"IN" => TokenType::TK_IN,
        b"INDEX" => TokenType::TK_INDEX,
        b"INDEXED" => TokenType::TK_INDEXED,
        b"INITIALLY" => TokenType::TK_INITIALLY,
        b"INNER" => TokenType::TK_JOIN_KW,
        b"INSERT" => TokenType::TK_INSERT,
        b"INSTEAD" => TokenType::TK_INSTEAD,
        b"INTERSECT" => TokenType::TK_INTERSECT,
        b"INTO" => TokenType::TK_INTO,
        b"IS" => TokenType::TK_IS,
        b"ISNULL" => TokenType::TK_ISNULL,
        b"JOIN" => TokenType::TK_JOIN,
        b"KEY" => TokenType::TK_KEY,
        b"LAST" => TokenType::TK_LAST,
        b"LEFT" => TokenType::TK_JOIN_KW,
        b"LIKE" => TokenType::TK_LIKE_KW,
        b"LIMIT" => TokenType::TK_LIMIT,
        b"MATCH" => TokenType::TK_MATCH,
        b"MATERIALIZED" => TokenType::TK_MATERIALIZED,
        b"NATURAL" => TokenType::TK_JOIN_KW,
        b"NO" => TokenType::TK_NO,
        b"NOT" => TokenType::TK_NOT,
        b"NOTHING" => TokenType::TK_NOTHING,
        b"NOTNULL" => TokenType::TK_NOTNULL,
        b"NULL" => TokenType::TK_NULL,
        b"NULLS" => TokenType::TK_NULLS,
        b"OF" => TokenType::TK_OF,
        b"OFFSET" => TokenType::TK_OFFSET,
        b"ON" => TokenType::TK_ON,
        b"OR" => TokenType::TK_OR,
        b"ORDER" => TokenType::TK_ORDER,
        b"OPTIMIZE" => TokenType::TK_OPTIMIZE,
        b"OTHERS" => TokenType::TK_OTHERS,
        b"OUTER" => TokenType::TK_JOIN_KW,
        b"OVER" => TokenType::TK_OVER,
        b"PARTITION" => TokenType::TK_PARTITION,
        b"PLAN" => TokenType::TK_PLAN,
        b"PRAGMA" => TokenType::TK_PRAGMA,
        b"PRECEDING" => TokenType::TK_PRECEDING,
        b"PRIMARY" => TokenType::TK_PRIMARY,
        b"QUERY" => TokenType::TK_QUERY,
        b"RAISE" => TokenType::TK_RAISE,
        b"RANGE" => TokenType::TK_RANGE,
        b"RECURSIVE" => TokenType::TK_RECURSIVE,
        b"REFERENCES" => TokenType::TK_REFERENCES,
        b"REGEXP" => TokenType::TK_LIKE_KW,
        b"REINDEX" => TokenType::TK_REINDEX,
        b"RELEASE" => TokenType::TK_RELEASE,
        b"RENAME" => TokenType::TK_RENAME,
        b"REPLACE" => TokenType::TK_REPLACE,
        b"RETURNING" => TokenType::TK_RETURNING,
        b"RESTRICT" => TokenType::TK_RESTRICT,
        b"RIGHT" => TokenType::TK_JOIN_KW,
        b"ROLLBACK" => TokenType::TK_ROLLBACK,
        b"ROW" => TokenType::TK_ROW,
        b"ROWS" => TokenType::TK_ROWS,
        b"SAVEPOINT" => TokenType::TK_SAVEPOINT,
        b"SELECT" => TokenType::TK_SELECT,
        b"SET" => TokenType::TK_SET,
        b"TABLE" => TokenType::TK_TABLE,
        b"TEMP" => TokenType::TK_TEMP,
        b"TEMPORARY" => TokenType::TK_TEMP,
        b"THEN" => TokenType::TK_THEN,
        b"TIES" => TokenType::TK_TIES,
        b"TO" => TokenType::TK_TO,
        b"TRANSACTION" => TokenType::TK_TRANSACTION,
        b"TRIGGER" => TokenType::TK_TRIGGER,
        b"TYPE" => TokenType::TK_TYPE,
        b"UNBOUNDED" => TokenType::TK_UNBOUNDED,
        b"UNION" => TokenType::TK_UNION,
        b"UNIQUE" => TokenType::TK_UNIQUE,
        b"UPDATE" => TokenType::TK_UPDATE,
        b"USING" => TokenType::TK_USING,
        b"VACUUM" => TokenType::TK_VACUUM,
        b"VALUES" => TokenType::TK_VALUES,
        b"VIEW" => TokenType::TK_VIEW,
        b"VIRTUAL" => TokenType::TK_VIRTUAL,
        b"WHEN" => TokenType::TK_WHEN,
        b"WHERE" => TokenType::TK_WHERE,
        b"WINDOW" => TokenType::TK_WINDOW,
        b"WITH" => TokenType::TK_WITH,
        b"WITHIN" => TokenType::TK_WITHIN,
        b"WITHOUT" => TokenType::TK_WITHOUT,
        _ => TokenType::TK_ID,
    })
}

#[inline(always)]
pub const fn is_identifier_start(b: u8) -> bool {
    b.is_ascii_uppercase() || b == b'_' || b.is_ascii_lowercase() || b > b'\x7F'
}

#[inline(always)]
pub const fn is_identifier_continue(b: u8) -> bool {
    b == b'$'
        || b.is_ascii_digit()
        || b.is_ascii_uppercase()
        || b == b'_'
        || b.is_ascii_lowercase()
        || b > b'\x7F'
}

#[derive(Clone, PartialEq, Eq, Debug)] // do not derive Copy for Token, just use .clone() when needed
pub struct Token<'a> {
    pub value: &'a [u8],
    pub token_type: TokenType, // None means Token is whitespaces or comments
}

impl<'a> Token<'a> {
    #[inline]
    pub const fn new(value: &'a [u8], token_type: TokenType) -> Self {
        Token { value, token_type }
    }
    #[inline]
    pub fn to_utf8(&self) -> String {
        String::from_utf8_lossy(self.as_bytes()).to_string()
    }
    /// # Safety
    /// Same as `String::from_utf8_unchecked`,
    /// the caller must ensure that token bytes are valid UTF-8.
    #[inline]
    pub unsafe fn to_utf8_unchecked(&self) -> String {
        String::from_utf8_unchecked(self.as_bytes().to_vec())
    }
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.value
    }
}

/// What one open `[` means for the text inside it (see `Lexer::brackets`).
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum BracketKind {
    /// `a[...]`: subscripts the value before it, and so may contain a
    /// top-level slice colon.
    Subscript,
    /// `ARRAY[...]`: an array literal. Its expressions are lexed normally
    /// but no slice colon can appear at its top level.
    ArrayLiteral,
    /// `[name]`: a quoted identifier. Its bytes are a name, not tokens:
    /// the parser skips them itself (see `Parser::parse_bracket_quoted_name`),
    /// and standalone lexing emits them as one raw token, so name text
    /// like `[` or `$x::y` cannot disturb the bracket state when the same
    /// SQL is re-tokenized for expanded SQL.
    QuotedName,
}

/// State for one open `[` (see `Lexer::brackets`).
#[derive(Clone, Copy)]
pub(crate) struct BracketFrame {
    kind: BracketKind,
    /// `(` groups currently open inside the bracket.
    parens: u32,
    /// `CASE ... END` constructs currently open inside the bracket.
    cases: u32,
    /// No slice colon can appear at the bracket's top level anymore:
    /// either the slice colon was already emitted (what follows is the
    /// upper bound), or the parser said this subscript takes a single
    /// index and no slice (an UPDATE ... SET assignment target).
    no_slice_colon_ahead: bool,
    /// Whether an explicit top-level slice colon sits somewhere ahead in
    /// this bracket. Filled by the lookahead in
    /// `Lexer::explicit_separator_ahead` the first time a "::" inside a
    /// parameter name forces the choice between splitting the name there
    /// and keeping it whole; None until then.
    explicit_colon_ahead: Option<bool>,
}

pub struct Lexer<'a> {
    pub(crate) offset: usize,
    pub(crate) input: &'a [u8],
    /// One entry per open `[`. A parameter name stops at "::" only where
    /// a slice colon could follow it: at the top level of a subscript,
    /// before that subscript's slice colon. So `arr[$lo::hi]` keeps its
    /// slice colon, while `arr[f($ns::var)]`, `ARRAY[$ns::idx]`,
    /// `arr[CASE WHEN b THEN $ns::idx END]` and `arr[1:$ns::hi]` keep
    /// their names whole. The state depends only on the token stream, so
    /// re-lexing the same text (for example for expanded SQL) reproduces
    /// the exact same tokens as parsing did.
    brackets: Vec<BracketFrame>,
    /// The last significant token can end an expression operand, so a `[`
    /// next subscripts that operand. Any other `[` can only start a
    /// quoted name (or an array literal, below): no grammar rule puts a
    /// subscript there.
    prev_can_end_operand: bool,
    /// The last significant token was the plain identifier `array` with
    /// no dot before it, so a `[` next starts an `ARRAY[...]` literal.
    prev_is_bare_array: bool,
    /// The last significant token was `.`. After a dot, `array` names a
    /// column (`t.array[...]` subscripts it), never an array literal.
    prev_was_dot: bool,
    /// True only in the throwaway lexer `explicit_separator_ahead` scans
    /// ahead with. The scan asks what the text means when parameter names
    /// stay greedy, so name splitting is off wholesale — which also keeps
    /// a scan from starting scans of its own.
    scanning_ahead: bool,
}

/// The lexer state the parser saves and restores when it rewinds the
/// input, and clears between statements (see `Parser::mark`,
/// `Parser::try_parse` and `Parser::next_cmd`).
#[derive(Clone)]
pub(crate) struct LexerContext {
    brackets: Vec<BracketFrame>,
    prev_can_end_operand: bool,
    prev_is_bare_array: bool,
    prev_was_dot: bool,
}

/// True for tokens that can end an expression operand, so that a `[`
/// directly after them subscripts the operand: literals, closing
/// delimiters, NULL, the postfix NULL tests, DEFAULT (an operand of its
/// own, `Expr::Default`), and every keyword the parser can read as an
/// identifier — the explicit rewrites in the parser's `next_token`
/// (`get_token`) plus `TokenType::fallback_id_if_ok`, which also covers
/// CASE's END and CURRENT_TIMESTAMP. Keep this in step with the parser:
/// a token it can end an operand with but that is missing here would
/// make a following `[` lex its contents as a raw quoted name. The
/// reverse mismatch is harmless: a quoted name directly after one of
/// these tokens (`DEFAULT [a]` in a column definition) still parses,
/// because the parser skips a quoted name's bytes itself without ever
/// lexing inside the misclassified bracket.
fn can_end_operand(token_type: TokenType) -> bool {
    use TokenType::*;
    matches!(
        token_type,
        TK_ID
            | TK_STRING
            | TK_BLOB
            | TK_INTEGER
            | TK_FLOAT
            | TK_VARIABLE
            | TK_RP
            | TK_RBRACKET
            | TK_NULL
            | TK_ISNULL
            | TK_NOTNULL
            | TK_DEFAULT
            | TK_INDEXED
            | TK_JOIN_KW
            | TK_UNION
            | TK_EXCEPT
            | TK_INTERSECT
            | TK_GENERATED
            | TK_WITHOUT
            | TK_COLUMNKW
            | TK_WINDOW
            | TK_FILTER
            | TK_OVER
            | TK_WITHIN
    ) || matches!(token_type.fallback_id_if_ok(), TK_ID)
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.next_token();
        if let Some(Ok(token)) = &item {
            self.update_bracket_context(token);
        }
        item
    }
}

impl<'a> Lexer<'a> {
    #[inline]
    fn next_token(&mut self) -> Option<Result<Token<'a>>> {
        // Inside a quoted name, everything up to the closing `]` is raw
        // name bytes: emit them as a single token instead of tokenizing
        // them. The parser never lexes here (it skips the name itself);
        // this keeps standalone re-lexing, as done for expanded SQL, in
        // step with the parser when the name holds text like `[` or `$x`.
        if self
            .brackets
            .last()
            .is_some_and(|frame| frame.kind == BracketKind::QuotedName)
        {
            match self.peek() {
                None => return None,
                Some(b']') => {}
                Some(_) => {
                    let start = self.offset;
                    while !matches!(self.peek(), None | Some(b']')) {
                        self.eat();
                    }
                    return Some(Ok(Token::new(
                        &self.input[start..self.offset],
                        TokenType::TK_ID,
                    )));
                }
            }
        }
        match self.peek() {
            None => None, // End of file
            Some(b) if b.is_ascii_whitespace() => Some(Ok(self.eat_white_space())),
            // matching logic
            Some(b) => match b {
                b'-' => Some(Ok(self.eat_minus_or_comment_or_ptr())),
                b'(' => Some(Ok(self.eat_one_token(TokenType::TK_LP))),
                b')' => Some(Ok(self.eat_one_token(TokenType::TK_RP))),
                b';' => Some(Ok(self.eat_one_token(TokenType::TK_SEMI))),
                b'+' => Some(Ok(self.eat_one_token(TokenType::TK_PLUS))),
                b'*' => Some(Ok(self.eat_one_token(TokenType::TK_STAR))),
                b'/' => Some(self.mark(|l| l.eat_slash_or_comment())),
                b'%' => Some(Ok(self.eat_one_token(TokenType::TK_REM))),
                b'=' => Some(Ok(self.eat_eq())),
                b'<' => Some(Ok(self.eat_le_or_ne_or_lshift_or_lt())),
                b'>' => Some(Ok(self.eat_ge_or_gt_or_rshift())),
                b'!' => Some(self.mark(|l| l.eat_ne())),
                b'|' => Some(Ok(self.eat_concat_or_bitor())),
                b',' => Some(Ok(self.eat_one_token(TokenType::TK_COMMA))),
                b'&' => Some(Ok(self.eat_overlap_or_bitand())),
                b'~' => Some(Ok(self.eat_one_token(TokenType::TK_BITNOT))),
                b'\'' | b'"' | b'`' => Some(self.mark(|l| l.eat_lit_or_id())),
                b'.' => Some(self.mark(|l| l.eat_dot_or_frac(false))),
                b'0'..=b'9' => Some(self.mark(|l| l.eat_number())),
                b'[' => Some(Ok(self.eat_one_token(TokenType::TK_LBRACKET))),
                b']' => Some(Ok(self.eat_one_token(TokenType::TK_RBRACKET))),
                b'@' => {
                    // @> is array contains operator; bare @ starts a variable
                    if self.input.get(self.offset + 1) == Some(&b'>') {
                        Some(Ok(self.eat_array_contains()))
                    } else {
                        Some(self.mark(|l| l.eat_var()))
                    }
                }
                b'?' | b'$' => Some(self.mark(|l| l.eat_var())),
                b'#' => {
                    let start = self.offset;
                    self.eat(); // consume '#'
                    self.eat_while(is_identifier_continue);
                    Some(Ok(Token::new(
                        &self.input[start..self.offset],
                        TokenType::TK_ILLEGAL,
                    )))
                }
                b':' => {
                    // `:name` is a named parameter, and so is `:::name` — the
                    // name may start with "::" sequences, as in SQLite. A `:`
                    // followed by anything else is a standalone colon (used in
                    // slice syntax); in particular `::` stays two colons so a
                    // slice like `arr[:lo::hi]` keeps its meaning.
                    if self.splits_param_names()
                        && self.input.get(self.offset + 1..self.offset + 4) == Some(b":::")
                    {
                        // In `a[1::::hi]`, the first colon is the slice
                        // separator and the remaining `:::hi` is the upper
                        // bound's global parameter.
                        return Some(Ok(self.eat_one_token(TokenType::TK_COLON)));
                    }
                    match (
                        self.input.get(self.offset + 1),
                        self.input.get(self.offset + 2),
                    ) {
                        (Some(&b), _) if is_identifier_start(b) => Some(self.mark(|l| l.eat_var())),
                        (Some(&b':'), Some(&b':')) => Some(self.mark(|l| l.eat_var())),
                        _ => Some(Ok(self.eat_one_token(TokenType::TK_COLON))),
                    }
                }
                b if is_identifier_start(b) => Some(self.mark(|l| l.eat_blob_or_id())),
                _ => Some(self.eat_unrecognized()),
            },
        }
    }
}

#[cold]
const fn cold() {}

impl<'a> Lexer<'a> {
    #[inline(always)]
    pub const fn new(input: &'a [u8]) -> Self {
        Lexer {
            input,
            offset: 0,
            brackets: Vec::new(),
            prev_can_end_operand: false,
            prev_is_bare_array: false,
            prev_was_dot: false,
            scanning_ahead: false,
        }
    }

    #[inline(always)]
    pub fn remaining(&self) -> &'a [u8] {
        self.input.get(self.offset..).unwrap_or(&[])
    }

    /// The parser calls this when it skips a `]` by advancing `offset`
    /// directly (bracket-quoted identifiers are scanned as raw bytes), so
    /// `brackets` stays balanced with the input. The skipped bytes and
    /// the `]` complete a quoted name, which is an operand — `[a][1:2]`
    /// slices the column a — so the lookbehind is set as if a `]` token
    /// had been lexed, which is also the state re-lexing the same text
    /// reaches after the name and its TK_RBRACKET.
    pub(crate) fn close_bracket(&mut self) {
        self.brackets.pop();
        self.prev_can_end_operand = true;
        self.prev_is_bare_array = false;
        self.prev_was_dot = false;
    }

    /// The parser calls this right after eating the `[` that opens an
    /// UPDATE `SET col[index] = ...` assignment target. That subscript
    /// holds a single index expression and never a slice, so no slice
    /// colon can follow a parameter name inside it and the name keeps
    /// its "::" (see `splits_param_names`).
    pub(crate) fn subscript_takes_no_slice(&mut self) {
        if let Some(frame) = self.brackets.last_mut() {
            frame.no_slice_colon_ahead = true;
        }
    }

    /// Snapshot the state that `update_bracket_context` maintains, so the
    /// parser can rewind the input and lex the same text again.
    pub(crate) fn save_context(&self) -> LexerContext {
        LexerContext {
            brackets: self.brackets.clone(),
            prev_can_end_operand: self.prev_can_end_operand,
            prev_is_bare_array: self.prev_is_bare_array,
            prev_was_dot: self.prev_was_dot,
        }
    }

    /// Restore a snapshot taken by `save_context`.
    pub(crate) fn restore_context(&mut self, saved: LexerContext) {
        self.brackets = saved.brackets;
        self.prev_can_end_operand = saved.prev_can_end_operand;
        self.prev_is_bare_array = saved.prev_is_bare_array;
        self.prev_was_dot = saved.prev_was_dot;
    }

    /// Reset the state that `update_bracket_context` maintains to what a
    /// fresh lexer starts with.
    pub(crate) fn clear_context(&mut self) {
        self.brackets.clear();
        self.prev_can_end_operand = false;
        self.prev_is_bare_array = false;
        self.prev_was_dot = false;
    }

    /// Keep `brackets` and the operand/`array` lookbehind in step with the
    /// token stream; called for every token `next` produces.
    fn update_bracket_context(&mut self, token: &Token<'a>) {
        match token.token_type {
            // Whitespace and comments separate nothing.
            TokenType::TK_NONE => return,
            TokenType::TK_LBRACKET => {
                let kind = if self.prev_is_bare_array {
                    BracketKind::ArrayLiteral
                } else if self.prev_can_end_operand {
                    BracketKind::Subscript
                } else {
                    BracketKind::QuotedName
                };
                self.brackets.push(BracketFrame {
                    kind,
                    parens: 0,
                    cases: 0,
                    no_slice_colon_ahead: false,
                    explicit_colon_ahead: None,
                });
            }
            TokenType::TK_RBRACKET => {
                self.brackets.pop();
            }
            TokenType::TK_LP => {
                if let Some(frame) = self.brackets.last_mut() {
                    frame.parens += 1;
                }
            }
            TokenType::TK_RP => {
                if let Some(frame) = self.brackets.last_mut() {
                    frame.parens = frame.parens.saturating_sub(1);
                }
            }
            TokenType::TK_CASE => {
                if let Some(frame) = self.brackets.last_mut() {
                    frame.cases += 1;
                }
            }
            TokenType::TK_END => {
                // END closes a CASE only after a complete branch expression.
                // In operand position — after THEN, ELSE, an operator — the
                // token is a column named `end` (TK_END can fall back to an
                // identifier), and the CASE stays open.
                if self.prev_can_end_operand {
                    if let Some(frame) = self.brackets.last_mut() {
                        frame.cases = frame.cases.saturating_sub(1);
                    }
                }
            }
            TokenType::TK_COLON => {
                if let Some(frame) = self.brackets.last_mut() {
                    if frame.kind == BracketKind::Subscript && frame.parens == 0 && frame.cases == 0
                    {
                        frame.no_slice_colon_ahead = true;
                    }
                }
            }
            _ => {}
        }
        self.prev_can_end_operand = can_end_operand(token.token_type);
        // The parser reads a `[` after the plain identifier `array` as an
        // ARRAY[...] literal and every other `[` after an operand as a
        // subscript; this raw-byte comparison is the same one the parser
        // makes. After a dot, `array` names a column (`t.array[1:2]` is a
        // slice). Quoted spellings ('array', `array`) keep their quotes in
        // the token value and never match, and a bracket-quoted [array]
        // never sits directly before a `[` because its own `]` comes
        // between.
        self.prev_is_bare_array = token.token_type == TokenType::TK_ID
            && !self.prev_was_dot
            && token.value.eq_ignore_ascii_case(b"array");
        self.prev_was_dot = token.token_type == TokenType::TK_DOT;
    }

    /// True where a parameter name must stop at "::" because a slice colon
    /// can follow it: at the top level of a subscript — outside any
    /// parentheses or CASE ... END opened inside it — before that
    /// subscript's slice colon.
    fn splits_param_names(&self) -> bool {
        !self.scanning_ahead
            && self.brackets.last().is_some_and(|frame| {
                frame.kind == BracketKind::Subscript
                    && frame.parens == 0
                    && frame.cases == 0
                    && !frame.no_slice_colon_ahead
            })
    }

    /// True when an explicit top-level slice colon sits ahead in the
    /// current subscript. This decides what a "::" inside a parameter
    /// name means: in `a[$lo::hi]` nothing else can separate the slice
    /// bounds, so the name splits there; in `a[$ns::lo : $hi]` the
    /// explicit colon is the separator, so the name keeps its "::" and
    /// stays whole. The scan lexes the rest of the bracket from the
    /// name's start with splitting off and looks for a colon token at
    /// this bracket's top level, outside parentheses and CASE ... END.
    /// The answer is cached in the bracket's frame: it cannot change
    /// while the top level still sits before its slice colon, and after
    /// the colon no name asks (`splits_param_names` is already false).
    fn explicit_separator_ahead(&mut self, name_start: usize) -> bool {
        let Some(frame) = self.brackets.last() else {
            return false;
        };
        if let Some(found) = frame.explicit_colon_ahead {
            return found;
        }
        let depth = self.brackets.len();
        let mut scratch = Lexer::new(self.input);
        scratch.offset = name_start;
        scratch.brackets.clone_from(&self.brackets);
        scratch.scanning_ahead = true;
        let mut found = false;
        while let Some(token) = scratch.next() {
            let Ok(token) = token else {
                // The bracket's text does not even lex; split as if the
                // scan had found nothing and let parsing surface the
                // error.
                break;
            };
            if scratch.brackets.len() < depth {
                // The subscript closed without a top-level colon.
                break;
            }
            if token.token_type == TokenType::TK_COLON && scratch.brackets.len() == depth {
                let frame = scratch.brackets.last().expect("length checked above");
                if frame.parens == 0 && frame.cases == 0 {
                    found = true;
                    break;
                }
            }
        }
        self.brackets
            .last_mut()
            .expect("checked non-empty above")
            .explicit_colon_ahead = Some(found);
        found
    }

    #[inline]
    pub fn mark<F, R>(&mut self, exc: F) -> Result<R>
    where
        F: FnOnce(&mut Self) -> Result<R>,
    {
        let start_offset = self.offset;
        let result = exc(self);
        if result.is_err() {
            self.offset = start_offset; // Reset to the start offset if an error occurs
        }
        result
    }

    /// Returns the current offset in the input without consuming.
    #[inline(always)]
    pub const fn peek(&self) -> Option<u8> {
        if self.offset < self.input.len() {
            Some(self.input[self.offset])
        } else {
            None // End of file
        }
    }

    /// Returns the current offset in the input and consumes it.
    #[inline(always)]
    pub const fn eat(&mut self) -> Option<u8> {
        if let Some(b) = self.peek() {
            self.offset += 1;
            Some(b)
        } else {
            None
        }
    }

    #[inline(always)]
    fn eat_and_assert<F>(&mut self, f: F)
    where
        F: Fn(u8) -> bool,
    {
        let _value = self.eat();
        debug_assert!(f(_value.unwrap()))
    }

    #[inline]
    // Eats up to but not including the specified byte, returns true if found
    fn eat_until(&mut self, byte: u8) -> bool {
        match memchr::memchr(byte, self.remaining()) {
            Some(pos) => {
                self.offset += pos;
                true
            }
            None => {
                cold();
                self.offset = self.input.len();
                false
            }
        }
    }

    #[inline]
    // Eats up to and including the specified byte, returns true if found
    fn eat_past(&mut self, byte: u8) -> bool {
        match memchr::memchr(byte, self.remaining()) {
            Some(pos) => {
                self.offset += pos + 1;
                true
            }
            None => {
                cold();
                self.offset = self.input.len();
                false
            }
        }
    }

    #[inline]
    fn eat_while<F>(&mut self, f: F)
    where
        F: Fn(u8) -> bool,
    {
        loop {
            if let Some(b) = self.peek() {
                if !f(b) {
                    cold();
                    return;
                }
            } else {
                cold();
                return;
            }

            self.eat();
        }
    }

    fn eat_while_number_digit(&mut self) -> Result<()> {
        loop {
            let start = self.offset;
            self.eat_while(|b| b.is_ascii_digit());
            match self.peek() {
                Some(b'_') => {
                    self.eat_and_assert(|b| b == b'_');

                    if start == self.offset {
                        // before the underscore, there was no digit
                        let token_text =
                            String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                        return Err(Error::BadNumber {
                            span: (start, self.offset - start).into(),
                            token_text,
                            offset: start,
                        });
                    }

                    match self.peek() {
                        Some(b) if b.is_ascii_digit() => continue, // Continue if next is a digit
                        _ => {
                            // after the underscore, there is no digit
                            let token_text =
                                String::from_utf8_lossy(&self.input[start..self.offset])
                                    .to_string();
                            return Err(Error::BadNumber {
                                span: (start, self.offset - start).into(),
                                token_text,
                                offset: start,
                            });
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    fn eat_while_number_hexdigit(&mut self) -> Result<()> {
        loop {
            let start = self.offset;
            self.eat_while(|b| b.is_ascii_hexdigit());
            match self.peek() {
                Some(b'_') => {
                    if start == self.offset {
                        // before the underscore, there was no digit
                        let token_text =
                            String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                        return Err(Error::BadNumber {
                            span: (start, self.offset - start).into(),
                            token_text,
                            offset: start,
                        });
                    }

                    self.eat_and_assert(|b| b == b'_');
                    match self.peek() {
                        Some(b) if b.is_ascii_hexdigit() => continue, // Continue if next is a digit
                        _ => {
                            // after the underscore, there is no digit
                            let token_text =
                                String::from_utf8_lossy(&self.input[start..self.offset])
                                    .to_string();
                            return Err(Error::BadNumber {
                                span: (start, self.offset - start).into(),
                                token_text,
                                offset: start,
                            });
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    #[inline]
    fn eat_one_token(&mut self, typ: TokenType) -> Token<'a> {
        debug_assert!(!self.remaining().is_empty());

        let tok = Token::new(self.remaining().get(..1).unwrap_or("".as_bytes()), typ);
        self.offset += 1;
        tok
    }

    #[inline]
    fn eat_white_space(&mut self) -> Token<'a> {
        let start = self.offset;
        self.eat_and_assert(|b| b.is_ascii_whitespace());
        self.eat_while(|b| b.is_ascii_whitespace());
        // This is whitespace
        Token::new(&self.input[start..self.offset], TokenType::TK_NONE)
    }

    fn eat_minus_or_comment_or_ptr(&mut self) -> Token<'a> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'-');

        match self.peek() {
            Some(b'-') => {
                self.eat_and_assert(|b| b == b'-');
                if self.eat_until(b'\n') {
                    self.eat_and_assert(|b| b == b'\n');
                }

                Token::new(&self.input[start..self.offset], TokenType::TK_NONE)
            }
            Some(b'>') => {
                self.eat_and_assert(|b| b == b'>');
                if self.peek() == Some(b'>') {
                    self.eat_and_assert(|b| b == b'>');
                }

                Token::new(&self.input[start..self.offset], TokenType::TK_PTR)
            }
            _ => Token::new(b"-", TokenType::TK_MINUS),
        }
    }

    fn eat_slash_or_comment(&mut self) -> Result<Token<'a>> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'/');
        match self.peek() {
            // C-style comments begin with "/*" and extend up to and
            // including the next "*/" character pair or until
            // the end of input, whichever comes first.
            Some(b'*') => {
                self.eat_and_assert(|b| b == b'*');
                loop {
                    if self.eat_past(b'*') {
                        match self.peek() {
                            Some(b'/') => {
                                self.eat_and_assert(|b| b == b'/');
                                break; // End of block comment
                            }
                            None => break,
                            _ => {}
                        }
                    } else {
                        cold();
                        break;
                    }
                }

                Ok(Token::new(
                    &self.input[start..self.offset],
                    TokenType::TK_NONE, // This is a comment
                ))
            }
            _ => Ok(Token::new(b"/", TokenType::TK_SLASH)),
        }
    }

    fn eat_eq(&mut self) -> Token<'a> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'=');
        if self.peek() == Some(b'=') {
            self.eat_and_assert(|b| b == b'=');
        }

        Token::new(&self.input[start..self.offset], TokenType::TK_EQ)
    }

    fn eat_le_or_ne_or_lshift_or_lt(&mut self) -> Token<'a> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'<');
        match self.peek() {
            Some(b'=') => {
                self.eat_and_assert(|b| b == b'=');
                Token::new(&self.input[start..self.offset], TokenType::TK_LE)
            }
            Some(b'<') => {
                self.eat_and_assert(|b| b == b'<');
                Token::new(&self.input[start..self.offset], TokenType::TK_LSHIFT)
            }
            Some(b'>') => {
                self.eat_and_assert(|b| b == b'>');
                Token::new(&self.input[start..self.offset], TokenType::TK_NE)
            }
            _ => Token::new(b"<", TokenType::TK_LT),
        }
    }

    fn eat_ge_or_gt_or_rshift(&mut self) -> Token<'a> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'>');
        match self.peek() {
            Some(b'=') => {
                self.eat_and_assert(|b| b == b'=');
                Token::new(&self.input[start..self.offset], TokenType::TK_GE)
            }
            Some(b'>') => {
                self.eat_and_assert(|b| b == b'>');
                Token::new(&self.input[start..self.offset], TokenType::TK_RSHIFT)
            }
            _ => Token::new(b">", TokenType::TK_GT),
        }
    }

    fn eat_ne(&mut self) -> Result<Token<'a>> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'!');
        match self.peek() {
            Some(b'=') => {
                self.eat_and_assert(|b| b == b'=');
            }
            _ => {
                let token_text =
                    String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                return Err(Error::ExpectedEqualsSign {
                    span: (start, self.offset - start).into(),
                    token_text,
                    offset: start,
                });
            }
        }

        Ok(Token::new(
            &self.input[start..self.offset],
            TokenType::TK_NE,
        ))
    }

    fn eat_concat_or_bitor(&mut self) -> Token<'a> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'|');

        if self.peek() == Some(b'|') {
            self.eat_and_assert(|b| b == b'|');
            return Token::new(&self.input[start..self.offset], TokenType::TK_CONCAT);
        }

        // Otherwise it is a bitwise OR operator
        Token::new(&self.input[start..self.offset], TokenType::TK_BITOR)
    }

    /// Tokenize `&&` (array overlap) or single `&` (bitwise AND).
    fn eat_overlap_or_bitand(&mut self) -> Token<'a> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'&');
        if self.peek() == Some(b'&') {
            self.eat_and_assert(|b| b == b'&');
            return Token::new(&self.input[start..self.offset], TokenType::TK_ARRAY_OVERLAP);
        }
        Token::new(&self.input[start..self.offset], TokenType::TK_BITAND)
    }

    /// Tokenize `@>` (array contains).
    fn eat_array_contains(&mut self) -> Token<'a> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'@');
        self.eat_and_assert(|b| b == b'>');
        Token::new(
            &self.input[start..self.offset],
            TokenType::TK_ARRAY_CONTAINS,
        )
    }

    fn eat_lit_or_id(&mut self) -> Result<Token<'a>> {
        let start = self.offset;
        let quote = self.eat().unwrap();
        debug_assert!(quote == b'\'' || quote == b'"' || quote == b'`');
        let tt = if quote == b'\'' {
            TokenType::TK_STRING
        } else {
            TokenType::TK_ID
        };

        loop {
            if self.eat_past(quote) {
                match self.peek() {
                    Some(b) if b == quote => {
                        self.eat_and_assert(|b| b == quote);
                        continue;
                    }
                    _ => break,
                }
            } else {
                cold();
                let token_text =
                    String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                return Err(Error::UnterminatedLiteral {
                    span: (start, self.offset - start).into(),
                    token_text,
                    offset: start,
                });
            }
        }

        Ok(Token::new(&self.input[start..self.offset], tt))
    }

    fn eat_dot_or_frac(&mut self, has_digit_prefix: bool) -> Result<Token<'a>> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'.');

        match self.peek() {
            Some(b)
                if b.is_ascii_digit() || (has_digit_prefix && b.eq_ignore_ascii_case(&b'e')) =>
            {
                self.eat_while_number_digit()?;
                match self.peek() {
                    Some(b'e') | Some(b'E') => {
                        _ = self.eat_expo()?;
                        Ok(Token::new(
                            &self.input[start..self.offset],
                            TokenType::TK_FLOAT,
                        ))
                    }
                    Some(b) if is_identifier_start(b) => {
                        let token_text =
                            String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                        Err(Error::BadFractionalPart {
                            span: (start, self.offset - start).into(),
                            token_text,
                            offset: start,
                        })
                    }
                    _ => Ok(Token::new(
                        &self.input[start..self.offset],
                        TokenType::TK_FLOAT,
                    )),
                }
            }
            _ => Ok(Token::new(
                &self.input[start..self.offset],
                TokenType::TK_DOT,
            )),
        }
    }

    fn eat_expo(&mut self) -> Result<Token<'a>> {
        let start = self.offset;
        self.eat_and_assert(|b| b == b'e' || b == b'E');
        match self.peek() {
            Some(b'+') | Some(b'-') => {
                self.eat_and_assert(|b| b == b'+' || b == b'-');
            }
            _ => {}
        }

        let start_num = self.offset;
        self.eat_while_number_digit()?;
        if start_num == self.offset {
            let token_text = String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
            return Err(Error::BadExponentPart {
                span: (start, self.offset - start).into(),
                token_text,
                offset: start,
            });
        }

        if self.peek().is_some() && is_identifier_start(self.peek().unwrap()) {
            let token_text = String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
            return Err(Error::BadExponentPart {
                span: (start, self.offset - start).into(),
                token_text,
                offset: start,
            });
        }

        // This is a number
        Ok(Token::new(
            &self.input[start..self.offset],
            TokenType::TK_FLOAT,
        ))
    }

    fn eat_number(&mut self) -> Result<Token<'a>> {
        let start = self.offset;
        let first_digit = self.eat().unwrap();
        debug_assert!(first_digit.is_ascii_digit());

        // hex int
        if first_digit == b'0' {
            match self.peek() {
                Some(b'x') | Some(b'X') => {
                    self.eat_and_assert(|b| b == b'x' || b == b'X');
                    let start_hex = self.offset;
                    self.eat_while_number_hexdigit()?;

                    if start_hex == self.offset {
                        let token_text =
                            String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                        return Err(Error::MalformedHexInteger {
                            span: (start, self.offset - start).into(),
                            token_text,
                            offset: start,
                        });
                    }

                    if self.peek().is_some() && is_identifier_start(self.peek().unwrap()) {
                        let token_text =
                            String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                        return Err(Error::BadNumber {
                            span: (start, self.offset - start).into(),
                            token_text,
                            offset: start,
                        });
                    }

                    return Ok(Token::new(
                        &self.input[start..self.offset],
                        TokenType::TK_INTEGER,
                    ));
                }
                _ => {}
            }
        }

        self.eat_while_number_digit()?;
        match self.peek() {
            Some(b'.') => {
                self.eat_dot_or_frac(true)?;
                Ok(Token::new(
                    &self.input[start..self.offset],
                    TokenType::TK_FLOAT,
                ))
            }
            Some(b'e') | Some(b'E') => {
                self.eat_expo()?;
                Ok(Token::new(
                    &self.input[start..self.offset],
                    TokenType::TK_FLOAT,
                ))
            }
            Some(b) if is_identifier_start(b) => {
                let token_text =
                    String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                Err(Error::BadNumber {
                    span: (start, self.offset - start).into(),
                    token_text,
                    offset: start,
                })
            }
            _ => Ok(Token::new(
                &self.input[start..self.offset],
                TokenType::TK_INTEGER,
            )),
        }
    }

    fn eat_var(&mut self) -> Result<Token<'a>> {
        let start = self.offset;
        let tok = self.eat().unwrap();
        debug_assert!(tok == b'?' || tok == b'$' || tok == b'@' || tok == b':');

        match tok {
            b'?' => {
                self.eat_while(|b| b.is_ascii_digit());

                Ok(Token::new(
                    &self.input[start..self.offset],
                    TokenType::TK_VARIABLE,
                ))
            }
            _ => {
                // Match SQLite's tokenizer: a named parameter is identifier
                // characters, plus two extras used by the TCL binding —
                // "::" sequences for namespace-qualified names ($::var,
                // $a::b) and one trailing "(...)" suffix for array elements
                // ($var(elem)). The suffix may not contain whitespace and
                // must be closed. At least one identifier character is
                // required overall.
                //
                // Exception: where a slice colon could follow the name (see
                // `splits_param_names`), a "::" after the name has begun
                // ends the token instead of continuing it, so a slice with
                // unspaced parameter bounds like `arr[$lo::hi]` still lexes
                // as `$lo`, `:`, `:hi` — unless an explicit top-level slice
                // colon sits ahead in the bracket (see
                // `explicit_separator_ahead`), as in `a[$ns::lo : $hi]`:
                // there that colon is the separator and the name stays
                // whole. Everywhere else — in parentheses within the
                // brackets, in ARRAY[...] literals, after the slice colon —
                // the name stays greedy, and a leading "::" is consumed
                // everywhere (`:::global`). SQLite has no subscript syntax
                // — its `[` starts a quoted identifier — so the non-greedy
                // context costs no compatibility.
                let mut n_id = 0usize;
                loop {
                    match self.peek() {
                        Some(b) if is_identifier_continue(b) => {
                            n_id += 1;
                            self.eat();
                        }
                        Some(b'(') if n_id > 0 => {
                            self.eat();
                            loop {
                                match self.peek() {
                                    Some(b')') => {
                                        self.eat();
                                        break;
                                    }
                                    Some(b) if !b.is_ascii_whitespace() => {
                                        self.eat();
                                    }
                                    _ => {
                                        // unclosed suffix, or whitespace
                                        // inside it
                                        let token_text = String::from_utf8_lossy(
                                            &self.input[start..self.offset],
                                        )
                                        .to_string();
                                        return Err(Error::BadVariableName {
                                            span: (start, self.offset - start).into(),
                                            token_text,
                                            offset: start,
                                        });
                                    }
                                }
                            }
                            break;
                        }
                        Some(b':') if self.input.get(self.offset + 1) == Some(&b':') => {
                            if n_id > 0
                                && self.splits_param_names()
                                && !self.explicit_separator_ahead(start)
                            {
                                // The "::" is the slice separator followed
                                // by the upper bound; the name ends here.
                                break;
                            }
                            self.eat();
                            self.eat();
                        }
                        _ => break,
                    }
                }

                if n_id == 0 {
                    let token_text =
                        String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                    return Err(Error::BadVariableName {
                        span: (start, self.offset - start).into(),
                        token_text,
                        offset: start,
                    });
                }

                Ok(Token::new(
                    &self.input[start..self.offset],
                    TokenType::TK_VARIABLE,
                ))
            }
        }
    }

    #[inline]
    fn eat_blob_or_id(&mut self) -> Result<Token<'a>> {
        let start = self.offset;
        let start_char = self.eat().unwrap();
        debug_assert!(is_identifier_start(start_char));

        match start_char {
            b'x' | b'X' if self.peek() == Some(b'\'') => {
                self.eat_and_assert(|b| b == b'\'');
                let start_hex = self.offset;
                self.eat_while(|b| b.is_ascii_hexdigit());

                match self.peek() {
                    Some(b'\'') => {
                        let end_hex = self.offset;
                        debug_assert!(end_hex >= start_hex);
                        self.eat_and_assert(|b| b == b'\'');

                        if ((end_hex - start_hex) & 1) != 0 {
                            let token_text =
                                String::from_utf8_lossy(&self.input[start..self.offset])
                                    .to_string();
                            return Err(Error::UnrecognizedToken {
                                span: (start, self.offset - start).into(),
                                token_text,
                                offset: start,
                            });
                        }
                        Ok(Token::new(
                            &self.input[start..self.offset],
                            TokenType::TK_BLOB,
                        ))
                    }
                    _ => {
                        let token_text =
                            String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
                        Err(Error::UnterminatedLiteral {
                            span: (start, self.offset - start).into(),
                            token_text,
                            offset: start,
                        })
                    }
                }
            }
            _ => {
                self.eat_while(is_identifier_continue);
                let result = &self.input[start..self.offset];
                Ok(Token::new(result, keyword_or_id_token(result)))
            }
        }
    }

    fn eat_unrecognized(&mut self) -> Result<Token<'a>> {
        let start = self.offset;
        self.eat_while(|b| !b.is_ascii_whitespace());
        let token_text = String::from_utf8_lossy(&self.input[start..self.offset]).to_string();
        Err(Error::UnrecognizedToken {
            span: (start, self.offset - start).into(),
            token_text,
            offset: start,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_lexer_one_tok() {
        let test_cases = vec![
            (b"    ".as_slice(), Token::new(b"    ", TokenType::TK_NONE)),
            (
                b"-- This is a comment\n".as_slice(),
                Token::new(b"-- This is a comment\n", TokenType::TK_NONE), // comment
            ),
            (b"-".as_slice(), Token::new(b"-", TokenType::TK_MINUS)),
            (b"->".as_slice(), Token::new(b"->", TokenType::TK_PTR)),
            (b"->>".as_slice(), Token::new(b"->>", TokenType::TK_PTR)),
            (b"(".as_slice(), Token::new(b"(", TokenType::TK_LP)),
            (b")".as_slice(), Token::new(b")", TokenType::TK_RP)),
            (b";".as_slice(), Token::new(b";", TokenType::TK_SEMI)),
            (b"+".as_slice(), Token::new(b"+", TokenType::TK_PLUS)),
            (b"*".as_slice(), Token::new(b"*", TokenType::TK_STAR)),
            (b"/".as_slice(), Token::new(b"/", TokenType::TK_SLASH)),
            (
                b"/* This is a block comment */".as_slice(),
                Token::new(b"/* This is a block comment */", TokenType::TK_NONE), // comment
            ),
            (
                b"/* This is a\n\n block comment */".as_slice(),
                Token::new(b"/* This is a\n\n block comment */", TokenType::TK_NONE), // comment
            ),
            (
                b"/* This is a** block* comment */".as_slice(),
                Token::new(b"/* This is a** block* comment */", TokenType::TK_NONE), // comment
            ),
            (b"=".as_slice(), Token::new(b"=", TokenType::TK_EQ)),
            (b"==".as_slice(), Token::new(b"==", TokenType::TK_EQ)),
            (b"<".as_slice(), Token::new(b"<", TokenType::TK_LT)),
            (b"<>".as_slice(), Token::new(b"<>", TokenType::TK_NE)),
            (b"<=".as_slice(), Token::new(b"<=", TokenType::TK_LE)),
            (b"<<".as_slice(), Token::new(b"<<", TokenType::TK_LSHIFT)),
            (b">".as_slice(), Token::new(b">", TokenType::TK_GT)),
            (b">=".as_slice(), Token::new(b">=", TokenType::TK_GE)),
            (b">>".as_slice(), Token::new(b">>", TokenType::TK_RSHIFT)),
            (b"!=".as_slice(), Token::new(b"!=", TokenType::TK_NE)),
            (b"|".as_slice(), Token::new(b"|", TokenType::TK_BITOR)),
            (b"||".as_slice(), Token::new(b"||", TokenType::TK_CONCAT)),
            (b",".as_slice(), Token::new(b",", TokenType::TK_COMMA)),
            (b"&".as_slice(), Token::new(b"&", TokenType::TK_BITAND)),
            (b"~".as_slice(), Token::new(b"~", TokenType::TK_BITNOT)),
            (
                b"'string'".as_slice(),
                Token::new(b"'string'", TokenType::TK_STRING),
            ),
            (
                b"`identifier`".as_slice(),
                Token::new(b"`identifier`", TokenType::TK_ID),
            ),
            (
                b"\"quoted string\"".as_slice(),
                Token::new(b"\"quoted string\"", TokenType::TK_ID),
            ),
            (
                b"\"\"\"triple \"\"quoted string\"\"\"".as_slice(),
                Token::new(b"\"\"\"triple \"\"quoted string\"\"\"", TokenType::TK_ID),
            ),
            (
                b"```triple ``quoted string```".as_slice(),
                Token::new(b"```triple ``quoted string```", TokenType::TK_ID),
            ),
            (
                b"'''triple ''quoted string'''".as_slice(),
                Token::new(b"'''triple ''quoted string'''", TokenType::TK_STRING),
            ),
            (b".".as_slice(), Token::new(b".", TokenType::TK_DOT)),
            (b".123".as_slice(), Token::new(b".123", TokenType::TK_FLOAT)),
            (b".456".as_slice(), Token::new(b".456", TokenType::TK_FLOAT)),
            (
                b".456e789".as_slice(),
                Token::new(b".456e789", TokenType::TK_FLOAT),
            ),
            (
                b".456E-789".as_slice(),
                Token::new(b".456E-789", TokenType::TK_FLOAT),
            ),
            (b"123".as_slice(), Token::new(b"123", TokenType::TK_INTEGER)),
            (
                b"9_223_372_036_854_775_807".as_slice(),
                Token::new(b"9_223_372_036_854_775_807", TokenType::TK_INTEGER),
            ),
            (
                b"123.456".as_slice(),
                Token::new(b"123.456", TokenType::TK_FLOAT),
            ),
            (
                b"123e456".as_slice(),
                Token::new(b"123e456", TokenType::TK_FLOAT),
            ),
            (
                b"123E-456".as_slice(),
                Token::new(b"123E-456", TokenType::TK_FLOAT),
            ),
            (
                b"0x1A3F".as_slice(),
                Token::new(b"0x1A3F", TokenType::TK_INTEGER),
            ),
            (
                b"0x1A3F_5678".as_slice(),
                Token::new(b"0x1A3F_5678", TokenType::TK_INTEGER),
            ),
            (
                b"0x1A3F_5678e9".as_slice(),
                Token::new(b"0x1A3F_5678e9", TokenType::TK_INTEGER),
            ),
            (b"[".as_slice(), Token::new(b"[", TokenType::TK_LBRACKET)),
            (b"]".as_slice(), Token::new(b"]", TokenType::TK_RBRACKET)),
            (
                b"?123".as_slice(),
                Token::new(b"?123", TokenType::TK_VARIABLE),
            ),
            (b"?".as_slice(), Token::new(b"?", TokenType::TK_VARIABLE)),
            (
                b"$var_name".as_slice(),
                Token::new(b"$var_name", TokenType::TK_VARIABLE),
            ),
            (
                b"@param".as_slice(),
                Token::new(b"@param", TokenType::TK_VARIABLE),
            ),
            (
                b":named_param".as_slice(),
                Token::new(b":named_param", TokenType::TK_VARIABLE),
            ),
            (
                b"$::global_var".as_slice(),
                Token::new(b"$::global_var", TokenType::TK_VARIABLE),
            ),
            (
                b"$ns::var".as_slice(),
                Token::new(b"$ns::var", TokenType::TK_VARIABLE),
            ),
            (
                b"@a::b::c".as_slice(),
                Token::new(b"@a::b::c", TokenType::TK_VARIABLE),
            ),
            (
                b":a::b::c".as_slice(),
                Token::new(b":a::b::c", TokenType::TK_VARIABLE),
            ),
            (
                b"@::global_var".as_slice(),
                Token::new(b"@::global_var", TokenType::TK_VARIABLE),
            ),
            (
                b":::global_var".as_slice(),
                Token::new(b":::global_var", TokenType::TK_VARIABLE),
            ),
            (
                b"$arr(elem)".as_slice(),
                Token::new(b"$arr(elem)", TokenType::TK_VARIABLE),
            ),
            (
                b"$arr(12x)".as_slice(),
                Token::new(b"$arr(12x)", TokenType::TK_VARIABLE),
            ),
            (
                b"x'1234567890abcdef'".as_slice(),
                Token::new(b"x'1234567890abcdef'", TokenType::TK_BLOB),
            ),
            (
                b"X'1234567890abcdef'".as_slice(),
                Token::new(b"X'1234567890abcdef'", TokenType::TK_BLOB),
            ),
            (b"x''".as_slice(), Token::new(b"x''", TokenType::TK_BLOB)),
            (b"X''".as_slice(), Token::new(b"X''", TokenType::TK_BLOB)),
            (
                b"wHeRe".as_slice(),
                Token::new(b"wHeRe", TokenType::TK_WHERE),
            ),
            (
                b"wHeRe123".as_slice(),
                Token::new(b"wHeRe123", TokenType::TK_ID),
            ),
            (
                b"wHeRe_123".as_slice(),
                Token::new(b"wHeRe_123", TokenType::TK_ID),
            ),
            // issue 2933
            (b"1.e5".as_slice(), Token::new(b"1.e5", TokenType::TK_FLOAT)),
            // issue 3425
            (b"/*".as_slice(), Token::new(b"/*", TokenType::TK_NONE)),
            (b"/**".as_slice(), Token::new(b"/**", TokenType::TK_NONE)),
        ];

        for (input, expected) in test_cases {
            let mut lexer = Lexer::new(input);
            let token = lexer.next().unwrap().unwrap();
            let expect_value = unsafe { expected.to_utf8_unchecked() };
            let got_value = unsafe { token.to_utf8_unchecked() };
            println!("Input: {input:?}, Expected: {expect_value:?}, Got: {got_value:?}");
            assert_eq!(got_value, expect_value);
            assert_eq!(token.token_type, expected.token_type);
        }
    }

    #[test]
    fn test_lexer_variables_inside_subscript() {
        // In a subscript's lower bound — the only place a slice colon can
        // follow — a "::" after a parameter name has begun ends the
        // parameter, so unspaced slice bounds keep their slice colon.
        // Everywhere else (parentheses inside the brackets, ARRAY[...]
        // literals, after the slice colon) the name is greedy, and a
        // leading "::" always belongs to the name.
        let cases: Vec<(&[u8], Vec<Token>)> = vec![
            (
                b"a[$lo::hi]".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"$lo", TokenType::TK_VARIABLE),
                    Token::new(b":", TokenType::TK_COLON),
                    Token::new(b":hi", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            (
                b"a[:lo::hi]".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b":lo", TokenType::TK_VARIABLE),
                    Token::new(b":", TokenType::TK_COLON),
                    Token::new(b":hi", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            (
                b"a[@lo::hi]".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"@lo", TokenType::TK_VARIABLE),
                    Token::new(b":", TokenType::TK_COLON),
                    Token::new(b":hi", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            // Parentheses inside the brackets suspend the splitting.
            (
                b"a[f($ns::var,0)]".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"f", TokenType::TK_ID),
                    Token::new(b"(", TokenType::TK_LP),
                    Token::new(b"$ns::var", TokenType::TK_VARIABLE),
                    Token::new(b",", TokenType::TK_COMMA),
                    Token::new(b"0", TokenType::TK_INTEGER),
                    Token::new(b")", TokenType::TK_RP),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            // A nested subscript inside those parentheses splits again, and
            // closing it restores the paren context.
            (
                b"a[f(b[$x::y],$p::q)]".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"f", TokenType::TK_ID),
                    Token::new(b"(", TokenType::TK_LP),
                    Token::new(b"b", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"$x", TokenType::TK_VARIABLE),
                    Token::new(b":", TokenType::TK_COLON),
                    Token::new(b":y", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                    Token::new(b",", TokenType::TK_COMMA),
                    Token::new(b"$p::q", TokenType::TK_VARIABLE),
                    Token::new(b")", TokenType::TK_RP),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            // An ARRAY[...] literal is not a subscript: no slice colon can
            // occur, so the name stays whole.
            (
                b"ARRAY[$ns::idx]".as_slice(),
                vec![
                    Token::new(b"ARRAY", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"$ns::idx", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            // After the slice colon, the upper bound cannot hold another
            // slice colon, so the name stays whole there too.
            (
                b"a[1:$ns::hi]".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"1", TokenType::TK_INTEGER),
                    Token::new(b":", TokenType::TK_COLON),
                    Token::new(b"$ns::hi", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            (
                b"a[1::::hi]".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"1", TokenType::TK_INTEGER),
                    Token::new(b":", TokenType::TK_COLON),
                    Token::new(b":::hi", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            // A subscript right after an ARRAY literal splits again.
            (
                b"ARRAY[1][$lo::hi]".as_slice(),
                vec![
                    Token::new(b"ARRAY", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"1", TokenType::TK_INTEGER),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"$lo", TokenType::TK_VARIABLE),
                    Token::new(b":", TokenType::TK_COLON),
                    Token::new(b":hi", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            // Leading "::" belongs to the name even in a lower bound.
            (
                b"a[$::g]".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"$::g", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                ],
            ),
            // After the bracket closes, names are greedy again.
            (
                b"a[$lo::hi]+$ns::var".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b"[", TokenType::TK_LBRACKET),
                    Token::new(b"$lo", TokenType::TK_VARIABLE),
                    Token::new(b":", TokenType::TK_COLON),
                    Token::new(b":hi", TokenType::TK_VARIABLE),
                    Token::new(b"]", TokenType::TK_RBRACKET),
                    Token::new(b"+", TokenType::TK_PLUS),
                    Token::new(b"$ns::var", TokenType::TK_VARIABLE),
                ],
            ),
        ];
        for (input, expected) in cases {
            let tokens: Vec<_> = Lexer::new(input).map(|t| t.unwrap()).collect();
            assert_eq!(
                tokens,
                expected,
                "input {:?}",
                String::from_utf8_lossy(input)
            );
        }
    }

    #[test]
    fn test_lexer_bad_variable_names() {
        // A parameter needs at least one identifier character, and a "(...)"
        // suffix must be closed with no whitespace inside — same rules as
        // SQLite's tokenizer.
        let bad_inputs: Vec<&[u8]> = vec![
            b"$",
            b"$(",
            b"$(elem)",
            b"$a(unclosed",
            b"$a(x y)",
            b"::::a",
        ];
        for input in bad_inputs {
            let mut lexer = Lexer::new(input);
            let result = lexer.next().unwrap();
            assert!(
                matches!(result, Err(Error::BadVariableName { .. })),
                "expected BadVariableName for {:?}, got {result:?}",
                String::from_utf8_lossy(input)
            );
        }
    }

    #[test]
    fn test_keyword_token() {
        let values = HashMap::from([
            ("ABORT", TokenType::TK_ABORT),
            ("ACTION", TokenType::TK_ACTION),
            ("ADD", TokenType::TK_ADD),
            ("AFTER", TokenType::TK_AFTER),
            ("ALL", TokenType::TK_ALL),
            ("ALTER", TokenType::TK_ALTER),
            ("ALWAYS", TokenType::TK_ALWAYS),
            ("ANALYZE", TokenType::TK_ANALYZE),
            ("AND", TokenType::TK_AND),
            ("AS", TokenType::TK_AS),
            ("ASC", TokenType::TK_ASC),
            ("ATTACH", TokenType::TK_ATTACH),
            ("AUTOINCREMENT", TokenType::TK_AUTOINCR),
            ("BEFORE", TokenType::TK_BEFORE),
            ("BEGIN", TokenType::TK_BEGIN),
            ("BETWEEN", TokenType::TK_BETWEEN),
            ("BY", TokenType::TK_BY),
            ("CASCADE", TokenType::TK_CASCADE),
            ("CASE", TokenType::TK_CASE),
            ("CAST", TokenType::TK_CAST),
            ("CHECK", TokenType::TK_CHECK),
            ("COLLATE", TokenType::TK_COLLATE),
            ("COLUMN", TokenType::TK_COLUMNKW),
            ("COMMIT", TokenType::TK_COMMIT),
            ("CONCURRENT", TokenType::TK_CONCURRENT),
            ("CONFLICT", TokenType::TK_CONFLICT),
            ("CONSTRAINT", TokenType::TK_CONSTRAINT),
            ("CREATE", TokenType::TK_CREATE),
            ("CROSS", TokenType::TK_JOIN_KW),
            ("CURRENT", TokenType::TK_CURRENT),
            ("CURRENT_DATE", TokenType::TK_CTIME_KW),
            ("CURRENT_TIME", TokenType::TK_CTIME_KW),
            ("CURRENT_TIMESTAMP", TokenType::TK_CTIME_KW),
            ("DATABASE", TokenType::TK_DATABASE),
            ("DEFAULT", TokenType::TK_DEFAULT),
            ("DEFERRABLE", TokenType::TK_DEFERRABLE),
            ("DEFERRED", TokenType::TK_DEFERRED),
            ("DELETE", TokenType::TK_DELETE),
            ("DESC", TokenType::TK_DESC),
            ("DETACH", TokenType::TK_DETACH),
            ("DISTINCT", TokenType::TK_DISTINCT),
            ("DO", TokenType::TK_DO),
            ("DROP", TokenType::TK_DROP),
            ("EACH", TokenType::TK_EACH),
            ("ELSE", TokenType::TK_ELSE),
            ("END", TokenType::TK_END),
            ("ESCAPE", TokenType::TK_ESCAPE),
            ("EXCEPT", TokenType::TK_EXCEPT),
            ("EXCLUDE", TokenType::TK_EXCLUDE),
            ("EXCLUSIVE", TokenType::TK_EXCLUSIVE),
            ("EXISTS", TokenType::TK_EXISTS),
            ("EXPLAIN", TokenType::TK_EXPLAIN),
            ("FAIL", TokenType::TK_FAIL),
            ("FILTER", TokenType::TK_FILTER),
            ("FIRST", TokenType::TK_FIRST),
            ("FOLLOWING", TokenType::TK_FOLLOWING),
            ("FOR", TokenType::TK_FOR),
            ("FOREIGN", TokenType::TK_FOREIGN),
            ("FROM", TokenType::TK_FROM),
            ("FULL", TokenType::TK_JOIN_KW),
            ("GENERATED", TokenType::TK_GENERATED),
            ("GLOB", TokenType::TK_LIKE_KW),
            ("GROUP", TokenType::TK_GROUP),
            ("GROUPS", TokenType::TK_GROUPS),
            ("HAVING", TokenType::TK_HAVING),
            ("IF", TokenType::TK_IF),
            ("IGNORE", TokenType::TK_IGNORE),
            ("IMMEDIATE", TokenType::TK_IMMEDIATE),
            ("IN", TokenType::TK_IN),
            ("INDEX", TokenType::TK_INDEX),
            ("INDEXED", TokenType::TK_INDEXED),
            ("INITIALLY", TokenType::TK_INITIALLY),
            ("INNER", TokenType::TK_JOIN_KW),
            ("INSERT", TokenType::TK_INSERT),
            ("INSTEAD", TokenType::TK_INSTEAD),
            ("INTERSECT", TokenType::TK_INTERSECT),
            ("INTO", TokenType::TK_INTO),
            ("IS", TokenType::TK_IS),
            ("ISNULL", TokenType::TK_ISNULL),
            ("JOIN", TokenType::TK_JOIN),
            ("KEY", TokenType::TK_KEY),
            ("LAST", TokenType::TK_LAST),
            ("LEFT", TokenType::TK_JOIN_KW),
            ("LIKE", TokenType::TK_LIKE_KW),
            ("LIMIT", TokenType::TK_LIMIT),
            ("MATCH", TokenType::TK_MATCH),
            ("MATERIALIZED", TokenType::TK_MATERIALIZED),
            ("NATURAL", TokenType::TK_JOIN_KW),
            ("NO", TokenType::TK_NO),
            ("NOT", TokenType::TK_NOT),
            ("NOTHING", TokenType::TK_NOTHING),
            ("NOTNULL", TokenType::TK_NOTNULL),
            ("NULL", TokenType::TK_NULL),
            ("NULLS", TokenType::TK_NULLS),
            ("OF", TokenType::TK_OF),
            ("OFFSET", TokenType::TK_OFFSET),
            ("ON", TokenType::TK_ON),
            ("OR", TokenType::TK_OR),
            ("ORDER", TokenType::TK_ORDER),
            ("OPTIMIZE", TokenType::TK_OPTIMIZE),
            ("OTHERS", TokenType::TK_OTHERS),
            ("OUTER", TokenType::TK_JOIN_KW),
            ("OVER", TokenType::TK_OVER),
            ("PARTITION", TokenType::TK_PARTITION),
            ("PLAN", TokenType::TK_PLAN),
            ("PRAGMA", TokenType::TK_PRAGMA),
            ("PRECEDING", TokenType::TK_PRECEDING),
            ("PRIMARY", TokenType::TK_PRIMARY),
            ("QUERY", TokenType::TK_QUERY),
            ("RAISE", TokenType::TK_RAISE),
            ("RANGE", TokenType::TK_RANGE),
            ("RECURSIVE", TokenType::TK_RECURSIVE),
            ("REFERENCES", TokenType::TK_REFERENCES),
            ("REGEXP", TokenType::TK_LIKE_KW),
            ("REINDEX", TokenType::TK_REINDEX),
            ("RELEASE", TokenType::TK_RELEASE),
            ("RENAME", TokenType::TK_RENAME),
            ("REPLACE", TokenType::TK_REPLACE),
            ("RETURNING", TokenType::TK_RETURNING),
            ("RESTRICT", TokenType::TK_RESTRICT),
            ("RIGHT", TokenType::TK_JOIN_KW),
            ("ROLLBACK", TokenType::TK_ROLLBACK),
            ("ROW", TokenType::TK_ROW),
            ("ROWS", TokenType::TK_ROWS),
            ("SAVEPOINT", TokenType::TK_SAVEPOINT),
            ("SELECT", TokenType::TK_SELECT),
            ("SET", TokenType::TK_SET),
            ("TABLE", TokenType::TK_TABLE),
            ("TEMP", TokenType::TK_TEMP),
            ("TEMPORARY", TokenType::TK_TEMP),
            ("THEN", TokenType::TK_THEN),
            ("TIES", TokenType::TK_TIES),
            ("TO", TokenType::TK_TO),
            ("TRANSACTION", TokenType::TK_TRANSACTION),
            ("TRIGGER", TokenType::TK_TRIGGER),
            ("UNBOUNDED", TokenType::TK_UNBOUNDED),
            ("UNION", TokenType::TK_UNION),
            ("UNIQUE", TokenType::TK_UNIQUE),
            ("UPDATE", TokenType::TK_UPDATE),
            ("USING", TokenType::TK_USING),
            ("VACUUM", TokenType::TK_VACUUM),
            ("VALUES", TokenType::TK_VALUES),
            ("VIEW", TokenType::TK_VIEW),
            ("VIRTUAL", TokenType::TK_VIRTUAL),
            ("WHEN", TokenType::TK_WHEN),
            ("WHERE", TokenType::TK_WHERE),
            ("WINDOW", TokenType::TK_WINDOW),
            ("WITH", TokenType::TK_WITH),
            ("WITHIN", TokenType::TK_WITHIN),
            ("WITHOUT", TokenType::TK_WITHOUT),
        ]);

        for (key, value) in &values {
            assert!(keyword_or_id_token(key.as_bytes()) == *value);
            assert!(keyword_or_id_token(key.as_bytes().to_ascii_lowercase().as_slice()) == *value);
        }

        assert_eq!(keyword_or_id_token(b""), TokenType::TK_ID);
        assert_eq!(keyword_or_id_token(b"wrong"), TokenType::TK_ID);
        assert_eq!(keyword_or_id_token(b"super wrong"), TokenType::TK_ID);
        assert_eq!(keyword_or_id_token(b"super_wrong"), TokenType::TK_ID);
        assert_eq!(
            keyword_or_id_token(b"aae26e78-3ba7-4627-8f8f-02623302495a"),
            TokenType::TK_ID
        );
        assert_eq!(
            keyword_or_id_token("Crème Brulée".as_bytes()),
            TokenType::TK_ID
        );
        assert_eq!(keyword_or_id_token("fróm".as_bytes()), TokenType::TK_ID);
    }

    #[test]
    fn test_lexer_multi_tok() {
        let test_cases = vec![
            (
                b"    SELECT 1".as_slice(),
                vec![
                    Token::new(b"    ", TokenType::TK_NONE),
                    Token::new(b"SELECT", TokenType::TK_SELECT),
                    Token::new(b" ", TokenType::TK_NONE),
                    Token::new(b"1", TokenType::TK_INTEGER),
                ],
            ),
            (
                b"INSERT INTO users VALUES (1,2,3)".as_slice(),
                vec![
                    Token::new(b"INSERT", TokenType::TK_INSERT),
                    Token::new(b" ", TokenType::TK_NONE),
                    Token::new(b"INTO", TokenType::TK_INTO),
                    Token::new(b" ", TokenType::TK_NONE),
                    Token::new(b"users", TokenType::TK_ID),
                    Token::new(b" ", TokenType::TK_NONE),
                    Token::new(b"VALUES", TokenType::TK_VALUES),
                    Token::new(b" ", TokenType::TK_NONE),
                    Token::new(b"(", TokenType::TK_LP),
                    Token::new(b"1", TokenType::TK_INTEGER),
                    Token::new(b",", TokenType::TK_COMMA),
                    Token::new(b"2", TokenType::TK_INTEGER),
                    Token::new(b",", TokenType::TK_COMMA),
                    Token::new(b"3", TokenType::TK_INTEGER),
                    Token::new(b")", TokenType::TK_RP),
                ],
            ),
            // issue 2933
            (
                b"u.email".as_slice(),
                vec![
                    Token::new(b"u", TokenType::TK_ID),
                    Token::new(b".", TokenType::TK_DOT),
                    Token::new(b"email", TokenType::TK_ID),
                ],
            ),
            // A bare "::" stays two colons; only ":::name" is a variable.
            (
                b"a::b".as_slice(),
                vec![
                    Token::new(b"a", TokenType::TK_ID),
                    Token::new(b":", TokenType::TK_COLON),
                    Token::new(b":b", TokenType::TK_VARIABLE),
                ],
            ),
        ];

        for (input, expected_tokens) in test_cases {
            let lexer = Lexer::new(input);
            let mut tokens = Vec::new();

            for token in lexer {
                tokens.push(token.unwrap());
            }

            assert_eq!(tokens.len(), expected_tokens.len());

            for (i, token) in tokens.iter().enumerate() {
                let expect_value =
                    unsafe { String::from_utf8_unchecked(expected_tokens[i].value.to_vec()) };
                let got_value = unsafe { String::from_utf8_unchecked(token.value.to_vec()) };
                assert_eq!(got_value, expect_value);
                assert_eq!(token.token_type, expected_tokens[i].token_type);
            }
        }
    }
}
