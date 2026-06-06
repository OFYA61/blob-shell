#[derive(Debug, Eq, PartialEq)]
pub enum TokenKind {
    Normal,
    SingleQuoteStr,
    DoubleQuoteStr,
}

impl TokenKind {
    #[inline]
    fn is_quote_str(&self) -> bool {
        match self {
            TokenKind::SingleQuoteStr | TokenKind::DoubleQuoteStr => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub lexeme: String,
    start: usize,
    end: usize,
    kind: TokenKind,
}

impl Token {
    pub fn new(lexeme: String, start: usize, end: usize, kind: TokenKind) -> Self {
        Self {
            lexeme,
            start,
            end,
            kind,
        }
    }
}

pub fn parse(command_raw: &str) -> Result<Vec<Token>, ()> {
    let mut tokens = tokenize(command_raw)?;
    join_concatenated_strings(&mut tokens);
    // TODO: parse and return a Vec of expressions to parse and execute instead of tokens
    Ok(tokens)
}

fn tokenize(command_raw: &str) -> Result<Vec<Token>, ()> {
    let mut tokens: Vec<Token> = Vec::new();

    let mut token_start: usize = 0;
    let mut c_iter = command_raw.chars().enumerate().peekable();

    while let Some((index, c)) = c_iter.next() {
        if c.is_whitespace() {
            let lexeme = &command_raw[token_start..index];
            if !lexeme.is_empty() {
                tokens.push(Token::new(
                    lexeme.to_owned(),
                    token_start,
                    index - 1,
                    TokenKind::Normal,
                ));
            }
            token_start = index + 1;
            continue;
        }

        if c == '\'' {
            let mut found = false;
            while let Some((index, c)) = c_iter.next() {
                if c == '\'' {
                    let lexeme = &command_raw[token_start + 1..index];
                    if !lexeme.is_empty() {
                        tokens.push(Token::new(
                            lexeme.to_owned(),
                            token_start,
                            index,
                            TokenKind::SingleQuoteStr,
                        ));
                    }
                    found = true;
                    token_start = index + 1;
                    break;
                }
            }
            if !found {
                eprintln!("Could not find closing \"'\" character");
                return Err(());
            }
        }

        if c == '"' {
            let mut found = false;
            while let Some((index, c)) = c_iter.next() {
                if c == '"' {
                    let lexeme = &command_raw[token_start + 1..index];
                    if !lexeme.is_empty() {
                        tokens.push(Token::new(
                            lexeme.to_owned(),
                            token_start,
                            index,
                            TokenKind::DoubleQuoteStr,
                        ));
                    }
                    found = true;
                    token_start = index + 1;
                    break;
                }
            }
            if !found {
                eprintln!("Could not find closing \"'\" character");
                return Err(());
            }
        }
    }

    let lexeme = &command_raw[token_start..command_raw.len()];
    if !lexeme.is_empty() {
        tokens.push(Token::new(
            lexeme.to_owned(),
            token_start,
            command_raw.len() - 1,
            TokenKind::DoubleQuoteStr,
        ));
    }

    Ok(tokens)
}

fn join_concatenated_strings(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        let mut new_token = None;

        if token.kind.is_quote_str() {
            if let Some(next_token) = tokens.get(i + 1)
                && token.end + 1 == next_token.start
            {
                let mut new_lexeme = token.lexeme.clone();
                new_lexeme.push_str(next_token.lexeme.as_str());
                new_token = Some(Token::new(
                    new_lexeme,
                    token.start,
                    next_token.end,
                    TokenKind::Normal,
                ));
            }
        }

        if let Some(new_token) = new_token {
            tokens.remove(i + 1);
            tokens.remove(i);
            tokens.insert(i, new_token);
        }

        i += 1;
    }
}
