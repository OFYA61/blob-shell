use std::collections::HashMap;
use std::sync::OnceLock;

fn single_char_tokens() -> &'static HashMap<char, TokenKind> {
    static MAP: OnceLock<HashMap<char, TokenKind>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut map: HashMap<char, TokenKind> = HashMap::new();
        map.insert('&', TokenKind::Ampersant);
        map.insert('|', TokenKind::Pipe);
        map
    })
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Word,

    /// Single quoted string
    LiteralString,
    /// Double quoted string
    FormatString,

    RedirectStdout,
    RedirectStdoutAppend,
    RedirectStderr,
    RedirectStderrAppend,

    Ampersant,
    Pipe,

    EOF,
}

impl TokenKind {
    #[inline]
    fn can_concat(&self) -> bool {
        match self {
            TokenKind::LiteralString | TokenKind::FormatString => true,
            _ => false,
        }
    }

    #[inline]
    fn blocks_concat(&self) -> bool {
        match self {
            TokenKind::Ampersant => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub lexeme: String,
    pub kind: TokenKind,
    start: usize,
    end: usize,
}

impl Token {
    fn new(lexeme: String, start: usize, end: usize, kind: TokenKind) -> Self {
        Self {
            lexeme,
            start,
            end,
            kind,
        }
    }
}

pub fn tokenize(command_raw: &str) -> Result<Vec<Token>, ()> {
    macro_rules! get_char {
        ($index:expr) => {
            unsafe { command_raw.chars().nth($index).unwrap_unchecked() }
        };
    }

    let mut tokens: Vec<Token> = Vec::new();

    let mut token_start: usize = 0;
    let mut index = 0;
    let mut lexeme = String::new();

    macro_rules! push_lexeme_as_token {
        () => {
            if !lexeme.is_empty() {
                let token_kind = match lexeme.as_str() {
                    ">" | "1>" => TokenKind::RedirectStdout,
                    ">>" | "1>>" => TokenKind::RedirectStdoutAppend,
                    "2>" => TokenKind::RedirectStderr,
                    "2>>" => TokenKind::RedirectStderrAppend,
                    _ => TokenKind::Word,
                };
                tokens.push(Token::new(
                    lexeme.clone(),
                    token_start,
                    index - 1,
                    token_kind,
                ));
            }
            token_start = index + 1;
            lexeme.clear();
        };
    }

    while index < command_raw.len() {
        let c = get_char!(index);

        if let Some(token_kind) = single_char_tokens().get(&c) {
            // Treat this as end of command, push the current lexeme to the list of tokens followed
            // up by the single char token
            push_lexeme_as_token!();
            tokens.push(Token::new(String::from(c), index, index, *token_kind));
        } else if c.is_whitespace() {
            push_lexeme_as_token!();
        } else if c == '\\' {
            index += 1;
            if let Some(c) = command_raw.chars().nth(index) {
                lexeme.push(c);
            } else {
                eprintln!("Expected escape character");
                return Err(());
            }
        } else if c == '\'' || c == '"' {
            if index > token_start {
                if !lexeme.is_empty() {
                    tokens.push(Token::new(
                        lexeme.clone(),
                        token_start,
                        index - 1,
                        TokenKind::Word,
                    ));
                }
                token_start = index;
                lexeme.clear();
                continue;
            }

            let closing_char = c;
            let enable_escape_chars = if c == '"' { true } else { false };
            let kind = if c == '"' {
                TokenKind::FormatString
            } else {
                TokenKind::LiteralString
            };
            let mut found = false;
            index += 1;
            while index < command_raw.len() {
                let c = get_char!(index);

                if c == '\\' && enable_escape_chars {
                    index += 1;
                    if let Some(c) = command_raw.chars().nth(index) {
                        if "\"\\".contains(c) {
                            lexeme.push(c);
                        } else {
                            lexeme.push('\\');
                            lexeme.push(c);
                        }
                        index += 1;
                    } else {
                        lexeme.push('\\');
                    }
                    continue;
                }

                if c == closing_char {
                    tokens.push(Token::new(lexeme.clone(), token_start, index, kind));
                    found = true;
                    token_start = index + 1;
                    lexeme.clear();
                    break;
                }
                lexeme.push(c);
                index += 1;
            }
            if !found {
                eprintln!("Could not find closing `{closing_char}` character");
                return Err(());
            }
        } else {
            lexeme.push(c);
        }
        index += 1;
    }

    if !lexeme.is_empty() {
        tokens.push(Token::new(
            lexeme.clone(),
            token_start,
            command_raw.len() - 1,
            TokenKind::Word,
        ));
    }

    join_concatenated_strings(&mut tokens);

    tokens.push(Token::new(
        lexeme.clone(),
        command_raw.len(),
        command_raw.len(),
        TokenKind::EOF,
    ));

    Ok(tokens)
}

fn join_concatenated_strings(tokens: &mut Vec<Token>) {
    let mut i = 1;
    while i < tokens.len() {
        let token = &tokens[i];
        let mut new_token = None;

        if let Some(prev_token) = tokens.get(i - 1)
            && (prev_token.kind.can_concat() || token.kind.can_concat())
            && !prev_token.kind.blocks_concat()
            && !token.kind.blocks_concat()
            && prev_token.end + 1 == token.start
        {
            let kind = if prev_token.kind == TokenKind::FormatString
                || token.kind == TokenKind::FormatString
            {
                TokenKind::FormatString
            } else {
                TokenKind::LiteralString
            };
            let mut new_lexeme = prev_token.lexeme.clone();
            new_lexeme.push_str(token.lexeme.as_str());
            new_token = Some(Token::new(new_lexeme, prev_token.start, token.end, kind));
        }

        if let Some(new_token) = new_token {
            tokens.remove(i);
            tokens.remove(i - 1);
            tokens.insert(i - 1, new_token);
            i -= 1;
        }

        i += 1;
    }
}
