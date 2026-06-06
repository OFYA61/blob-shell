#[derive(Debug, Eq, PartialEq)]
pub enum TokenKind {
    Normal,
    SingleQuoteStr,
    DoubleQuoteStr,
    Joined,
}

impl TokenKind {
    #[inline]
    fn can_concat(&self) -> bool {
        match self {
            TokenKind::SingleQuoteStr | TokenKind::DoubleQuoteStr | TokenKind::Joined => true,
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
    let mut index = 0;
    while index < command_raw.len() {
        let c = unsafe { command_raw.chars().nth(index).unwrap_unchecked() };

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
            index += 1;
            continue;
        }

        if c == '\'' || c == '"' {
            if index > token_start {
                let lexeme = &command_raw[token_start..index];
                if !lexeme.is_empty() {
                    tokens.push(Token::new(
                        lexeme.to_owned(),
                        token_start,
                        index - 1,
                        TokenKind::Normal,
                    ));
                }
                token_start = index;
                continue;
            }

            let token_kind = match c {
                '\'' => TokenKind::SingleQuoteStr,
                '"' => TokenKind::DoubleQuoteStr,
                _ => unreachable!(),
            };

            let closing_char = c;
            let mut found = false;
            index += 1;
            while index < command_raw.len() {
                let c = unsafe { command_raw.chars().nth(index).unwrap_unchecked() };

                if c == closing_char {
                    let lexeme = &command_raw[token_start + 1..index];
                    tokens.push(Token::new(
                        lexeme.to_owned(),
                        token_start,
                        index,
                        token_kind,
                    ));
                    found = true;
                    token_start = index + 1;
                    break;
                }
                index += 1;
            }
            if !found {
                eprintln!("Could not find closing `{closing_char}` character");
                return Err(());
            }
        }

        index += 1;
    }

    let lexeme = &command_raw[token_start..command_raw.len()];
    tokens.push(Token::new(
        lexeme.to_owned(),
        token_start,
        command_raw.len() - 1,
        TokenKind::Normal,
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
            && prev_token.end + 1 == token.start
        {
            let mut new_lexeme = prev_token.lexeme.clone();
            new_lexeme.push_str(token.lexeme.as_str());
            new_token = Some(Token::new(
                new_lexeme,
                prev_token.start,
                token.end,
                TokenKind::Joined,
            ));
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
