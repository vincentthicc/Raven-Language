use std::fs;
use crossbeam_channel::Sender;
use lsp_server::{Message, RequestId, Response};
use lsp_types::{SemanticToken, SemanticTokens, SemanticTokensResult, Url};
use parser::tokens::tokenizer::Tokenizer;
use parser::tokens::tokens::{Token, TokenTypes};
use urlencoding::decode;

pub async fn parse_semantic_tokens(id: RequestId, file: Url, sender: Sender<Message>) {
    let mut url = decode(file.path()).unwrap().to_string();
    url.remove(0);

    let buffer = fs::read(url).unwrap();
    let mut tokenizer = Tokenizer::new(buffer.as_slice());
    let mut tokens = Vec::new();
    loop {
        tokens.push(tokenizer.next());
        if tokens.last().unwrap().token_type == TokenTypes::EOF {
            break
        }
    }

    let mut last: Option<Token> = None;
    let data = tokens.into_iter().map(|mut token| {
        if token.start.0 != token.end.0 {
            token.start = (token.end.0, 0);
        }
        let delta_line = (token.start.0 - 1) - last.clone().map(|inner| inner.start.0 - 1).unwrap_or(0);
        eprintln!("Line ({}, {}) to ({}, {}) for {:?} ({})", token.start.0, token.start.1, token.end.0, token.end.1, token.token_type,
                  token.start_offset - last.clone().map(|inner| inner.end_offset).unwrap_or(0));
        let temp = SemanticToken {
            delta_line,
            delta_start: if delta_line == 0 {
                (token.start_offset - last.clone().map(|inner| inner.start_offset).unwrap_or(0)) / 2
            } else {
                0
            } as u32 * 2,
            length: (token.end_offset - token.start_offset) as u32,
            token_type: get_token(&token.token_type),
            token_modifiers_bitset: 0,
        };
        last = Some(token);
        temp
    }).collect::<Vec<_>>();
    let result = Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data,
    }));
    let result = serde_json::to_value(&result).unwrap();
    let resp = Response { id, result: Some(result), error: None };
    sender.send(Message::Response(resp)).unwrap();
}

fn get_token(token_type: &TokenTypes) -> u32 {
    let temp = match token_type {
        TokenTypes::Identifier => 1,
        TokenTypes::Variable => 9,
        TokenTypes::ImportStart | TokenTypes::Return | TokenTypes::New | TokenTypes::Modifier |
        TokenTypes::ReturnType | TokenTypes::FunctionStart => 15,
        TokenTypes::Comment => 17,
        TokenTypes::StringStart | TokenTypes::StringEnd | TokenTypes::StringEscape => 18,
        TokenTypes::ReturnTypeArrow | TokenTypes::ImportEnd => 100,
        _ => (token_type.clone() as u32) % 21
    };
    eprintln!("{:?}: {}", token_type, temp);
    return temp;
}