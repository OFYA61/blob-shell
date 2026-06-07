mod tokenizer;

use self::tokenizer::Token;
use self::tokenizer::TokenKind;

pub fn parse_to_ast(command_raw: &str) -> Result<Vec<Expr>, ()> {
    let tokens = tokenizer::tokenize(command_raw)?;
    Ok(Parser::new(tokens).parse().map_err(|_| ())?)
}

#[derive(Debug)]
pub enum Expr {
    Command { exec: ExprArg, args: Vec<ExprArg> },
}

#[derive(Debug)]
pub enum ExprArg {
    Word(String),
    Literal(String),
    Format(String),
}

impl ExprArg {
    pub fn process(&self) -> &str {
        match self {
            ExprArg::Word(s) => s,
            ExprArg::Literal(s) => s,
            ExprArg::Format(s) => s,
        }
    }

    fn from_token(token: &Token) -> Self {
        match token.kind {
            TokenKind::Word => Self::Word(token.lexeme.clone()),
            TokenKind::LiteralString => Self::Literal(token.lexeme.clone()),
            TokenKind::FormatString => Self::Format(token.lexeme.clone()),
            TokenKind::EOF => unreachable!("Should never try and translate EOF to ExprArg"),
        }
    }
}

/// Top down parser
///
/// Parsing rules
/// ```ignore
/// command -> expr_arg+ EOF
/// expr_arg -> WORD | LITERAL_STRING | FORMAT_STRING
/// ```
struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

#[derive(Debug)]
enum ParserError {
    EOF,
    WrongToken,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse(&mut self) -> Result<Vec<Expr>, ParserError> {
        Ok(vec![self.command()?])
    }

    fn command(&mut self) -> Result<Expr, ParserError> {
        let exec = ExprArg::from_token(self.consume_any(vec![
            TokenKind::Word,
            TokenKind::LiteralString,
            TokenKind::FormatString,
        ])?);

        let mut args: Vec<ExprArg> = vec![];
        while self.peek_token()?.kind != TokenKind::EOF {
            args.push(self.expr_arg()?);
        }

        self.consume(TokenKind::EOF)?;
        Ok(Expr::Command { exec, args })
    }

    fn expr_arg(&mut self) -> Result<ExprArg, ParserError> {
        Ok(ExprArg::from_token(self.consume_any(vec![
            TokenKind::Word,
            TokenKind::LiteralString,
            TokenKind::FormatString,
        ])?))
    }

    fn next_token(&mut self) -> Result<&Token, ParserError> {
        if let Some(token) = self.tokens.get(self.index) {
            self.index += 1;
            return Ok(token);
        }
        Err(ParserError::EOF)
    }

    fn peek_token(&mut self) -> Result<&Token, ParserError> {
        if let Some(token) = self.tokens.get(self.index) {
            return Ok(token);
        }
        Err(ParserError::EOF)
    }

    fn consume(&mut self, token_kind: TokenKind) -> Result<&Token, ParserError> {
        let token = self.next_token()?;
        if token.kind != token_kind {
            eprintln!(
                "Expected token of type {:?} but got {:?}",
                token_kind, token.kind
            );
            return Err(ParserError::WrongToken);
        }
        Ok(token)
    }

    fn consume_any(&mut self, token_kinds: Vec<TokenKind>) -> Result<&Token, ParserError> {
        let token = self.next_token()?;
        if !token_kinds.contains(&token.kind) {
            eprintln!(
                "Expected token of types {:?} but got {:?}",
                token_kinds, token.kind
            );
            return Err(ParserError::WrongToken);
        }
        Ok(token)
    }
}
