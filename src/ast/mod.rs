mod tokenizer;

use std::path::Path;

use tokio::fs::File;
use tokio::fs::OpenOptions;

use self::tokenizer::Token;
use self::tokenizer::TokenKind;

pub fn parse(command_raw: &str) -> Result<Vec<Expr>, ()> {
    let tokens = tokenizer::tokenize(command_raw)?;
    Ok(Parser::new(tokens).parse().map_err(|_| ())?)
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub is_background: bool,
}

#[derive(Debug)]
pub enum ExprKind {
    Command(ExprCommand),
    PipedCommands(ExprPipedCommands),
}

#[derive(Debug)]
pub struct ExprCommand {
    pub exec: ExprArg,
    pub args: Vec<ExprArg>,
    pub redirects: Vec<ExprRedirect>,
}

#[derive(Debug)]
pub struct ExprPipedCommands {
    pub commands: Vec<ExprCommand>,
}

#[derive(Debug, Clone)]
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
            _ => unreachable!("Should never translate {:?} to ExprArg", token.kind),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ExprRedirectKind {
    Stdout,
    StdoutAppend,
    Stderr,
    StderrAppend,
}

impl ExprRedirectKind {
    fn from_token(token: &Token) -> Self {
        match token.kind {
            TokenKind::RedirectStdout => Self::Stdout,
            TokenKind::RedirectStdoutAppend => Self::StdoutAppend,
            TokenKind::RedirectStderr => Self::Stderr,
            TokenKind::RedirectStderrAppend => Self::StderrAppend,
            _ => unreachable!(
                "Should never translate {:?} to ExprRedirectKind",
                token.kind
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExprRedirect {
    kind: ExprRedirectKind,
    arg: ExprArg,
}

impl ExprRedirect {
    pub fn is_stdout(&self) -> bool {
        match self.kind {
            ExprRedirectKind::Stdout | ExprRedirectKind::StdoutAppend => true,
            _ => false,
        }
    }

    pub fn is_stderr(&self) -> bool {
        match self.kind {
            ExprRedirectKind::Stderr | ExprRedirectKind::StderrAppend => true,
            _ => false,
        }
    }

    fn is_append(&self) -> bool {
        match self.kind {
            ExprRedirectKind::StdoutAppend | ExprRedirectKind::StderrAppend => true,
            _ => false,
        }
    }

    pub async fn open_file(&self) -> File {
        // TODO: print out error message instead of panicing when file cannot be opened
        let path = Path::new(self.arg.process());
        if self.is_append() {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await
                .expect("Failed to open file in append mode")
        } else {
            File::create(path).await.expect("Failed to open file")
        }
    }
}

/// Top down parser
///
/// Parsing rules
/// ```ignore
/// root -> piped EOF
/// piped -> command (PIPE command)+ AMPERSANT?
/// command -> expr_arg* expr_redirect*
/// expr_arg -> WORD | LITERAL_STRING | FORMAT_STRING
/// expr_redirect -> (REDIRECT_STDOUT | REDIRECT_STDERR) expr_arg
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
        let piped = self.piped()?;
        self.consume(TokenKind::EOF)?;
        Ok(vec![piped])
    }

    fn piped(&mut self) -> Result<Expr, ParserError> {
        let command = self.command()?;

        if self.match_exact(TokenKind::Ampersant)? {
            self.consume(TokenKind::Ampersant)?;
            return Ok(Expr {
                kind: ExprKind::Command(command),
                is_background: true,
            });
        } else if self.match_exact(TokenKind::Pipe)? {
            let mut commands: Vec<ExprCommand> = Vec::new();
            commands.push(command);
            while self.match_exact(TokenKind::Pipe)? {
                self.consume(TokenKind::Pipe)?;
                commands.push(self.command()?);
            }

            let is_background = if self.match_exact(TokenKind::Ampersant)? {
                self.consume(TokenKind::Ampersant)?;
                true
            } else {
                false
            };

            return Ok(Expr {
                kind: ExprKind::PipedCommands(ExprPipedCommands { commands }),
                is_background,
            });
        }

        Ok(Expr {
            kind: ExprKind::Command(command),
            is_background: false,
        })
    }

    fn command(&mut self) -> Result<ExprCommand, ParserError> {
        let exec = ExprArg::from_token(self.consume_any(vec![
            TokenKind::Word,
            TokenKind::LiteralString,
            TokenKind::FormatString,
        ])?);

        let mut args: Vec<ExprArg> = vec![];
        while self.match_any(vec![
            TokenKind::Word,
            TokenKind::LiteralString,
            TokenKind::FormatString,
        ])? {
            args.push(self.expr_arg()?);
        }

        let mut redirects: Vec<ExprRedirect> = vec![];
        while self.match_any(vec![
            TokenKind::RedirectStdout,
            TokenKind::RedirectStdoutAppend,
            TokenKind::RedirectStderr,
            TokenKind::RedirectStderrAppend,
        ])? {
            redirects.push(self.expr_redirect()?);
        }

        Ok(ExprCommand {
            exec,
            args,
            redirects,
        })
    }

    fn expr_arg(&mut self) -> Result<ExprArg, ParserError> {
        Ok(ExprArg::from_token(self.consume_any(vec![
            TokenKind::Word,
            TokenKind::LiteralString,
            TokenKind::FormatString,
        ])?))
    }

    fn expr_redirect(&mut self) -> Result<ExprRedirect, ParserError> {
        let kind = ExprRedirectKind::from_token(self.consume_any(vec![
            TokenKind::RedirectStdout,
            TokenKind::RedirectStdoutAppend,
            TokenKind::RedirectStderr,
            TokenKind::RedirectStderrAppend,
        ])?);
        let arg = self.expr_arg()?;
        Ok(ExprRedirect { kind, arg })
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

    fn match_exact(&mut self, token_kind: TokenKind) -> Result<bool, ParserError> {
        let token = self.peek_token()?;
        if token.kind != token_kind {
            return Ok(false);
        }
        Ok(true)
    }

    fn match_any(&mut self, token_kinds: Vec<TokenKind>) -> Result<bool, ParserError> {
        let token = self.peek_token()?;
        if !token_kinds.contains(&token.kind) {
            return Ok(false);
        }
        Ok(true)
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
