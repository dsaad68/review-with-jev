use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Symbol(u32);

#[derive(Default)]
struct Interner {
    map: HashMap<String, Symbol>,
    names: Vec<String>,
}

impl Interner {
    fn intern(&mut self, text: &str) -> Symbol {
        if let Some(&sym) = self.map.get(text) {
            return sym;
        }
        let sym = Symbol(self.names.len() as u32);
        self.names.push(text.to_owned());
        self.map.insert(text.to_owned(), sym);
        sym
    }

    fn resolve(&self, sym: Symbol) -> &str {
        &self.names[sym.0 as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenKind {
    Ident(Symbol),
    Str(Symbol),
    Number(f64),
    Punct(char),
}

#[derive(Debug, Clone, Copy)]
struct Span { start: usize, end: usize }

#[derive(Debug, Clone, Copy)]
struct Token {
    kind: TokenKind,
    span: Span,
}

#[derive(Debug)]
enum LexError { Unterminated(usize), Unexpected(char, usize) }

fn unescape(raw: &str) -> String {
    if !raw.contains('\\') {
        return raw.to_string();
    }
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

struct Lexer<'src, 'i> {
    src: &'src str,
    pos: usize,
    interner: &'i mut Interner,
}

impl<'src, 'i> Lexer<'src, 'i> {
    fn new(src: &'src str, interner: &'i mut Interner) -> Self {
        Lexer { src, pos: 0, interner }
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn eat_while<F: Fn(char) -> bool>(&mut self, pred: F) -> &'src str {
        let start = self.pos;
        while self.peek().map_or(false, &pred) {
            self.bump();
        }
        &self.src[start..self.pos]
    }

    fn skip_trivia(&mut self) {
        loop {
            self.eat_while(char::is_whitespace);
            if self.peek() == Some('#') {
                self.eat_while(|c| c != '\n');
            } else {
                break;
            }
        }
    }

    fn lex_string(&mut self, start: usize) -> Result<TokenKind, LexError> {
        let body_start = self.pos;
        let mut escaped = false;
        loop {
            match self.bump() {
                None => return Err(LexError::Unterminated(start)),
                Some('\\') if !escaped => escaped = true,
                Some('"') if !escaped => break,
                Some(_) => escaped = false,
            }
        }
        let raw = &self.src[body_start..self.pos - 1];
        let text = unescape(raw);
        Ok(TokenKind::Str(self.interner.intern(&text)))
    }
}

impl<'src, 'i> Iterator for Lexer<'src, 'i> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_trivia();
        let start = self.pos;
        let c = self.bump()?;
        let kind = if c.is_alphabetic() || c == '_' {
            self.eat_while(|d| d.is_alphanumeric() || d == '_');
            Ok(TokenKind::Ident(self.interner.intern(&self.src[start..self.pos])))
        } else if c.is_ascii_digit() {
            self.eat_while(|d| d.is_ascii_digit() || d == '.');
            Ok(TokenKind::Number(self.src[start..self.pos].parse().unwrap_or(f64::NAN)))
        } else if c == '"' {
            self.lex_string(start)
        } else if "+-*/=(){},;".contains(c) {
            Ok(TokenKind::Punct(c))
        } else {
            Err(LexError::Unexpected(c, start))
        };
        Some(kind.map(|kind| Token { kind, span: Span { start, end: self.pos } }))
    }
}

fn main() {
    let source = r#"
        # greeting module
        let name = "world";
        let banner = "hello\t\"friend\"";
        print(name, banner, 42, 3.5);
        let name2 = "world";
        print(name2 @ 1);
    "#;
    let mut interner = Interner::default();
    let mut tokens = Vec::new();
    {
        let lexer = Lexer::new(source, &mut interner);
        for result in lexer {
            match result {
                Ok(tok) => tokens.push(tok),
                Err(LexError::Unterminated(at)) => eprintln!("unterminated string at {}", at),
                Err(LexError::Unexpected(c, at)) => eprintln!("unexpected {:?} at {}", c, at),
            }
        }
    }
    for tok in &tokens {
        match tok.kind {
            TokenKind::Ident(s) | TokenKind::Str(s) => println!("{:>3}..{:<3} {:?}", tok.span.start, tok.span.end, interner.resolve(s)),
            other => println!("{:>3}..{:<3} {:?}", tok.span.start, tok.span.end, other),
        }
    }
    println!("tokens={} symbols={}", tokens.len(), interner.names.len());
}
