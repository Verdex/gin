
mod array_pattern;
mod data_pattern;

mod data;
mod parsing;

fn main() {

    let input = "";
    let token_results = parsing::tokenizer::tokenize(input);
    match token_results {
        Ok(tokens) => {
            let _ast_result = parsing::parser::parse(input, tokens);
        },
        Err(message) => println!("{}", message),
    }
}
