pub mod tokenizer;

pub fn parse(command_raw: &str) -> Result<Vec<tokenizer::Token>, ()> {
    let tokens = tokenizer::tokenize(command_raw)?;
    Ok(tokens)
}
