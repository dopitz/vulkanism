use proc_macro::TokenStream;
use proc_macro::TokenTree;

type Tokens = dyn Iterator<Item = TokenTree>;

pub fn trim_quote(s: &str) -> String {
  s.chars().skip(1).take(s.len() - 2).collect()
}

pub fn parse_string(tokens: &mut Tokens) -> String {
  trim_quote(&parse(tokens))
}

pub fn parse(tokens: &mut Tokens) -> String {
  tokens
    .take_while(|tok| match tok {
      TokenTree::Punct(p) => p.to_string() != ",",
      _ => true,
    }).fold(String::new(), |s, tok| format!("{}{}", s, tok.to_string()))
}

pub fn parse_group<F, T>(tokens: &mut Tokens, f: F) -> Result<T, String>
where
  F: Fn(TokenStream) -> Result<T, String>,
{
  match tokens.next() {
    Some(group) => match group {
      TokenTree::Group(g) => f(g.stream().clone()),
      TokenTree::Ident(_) => Err("expected TokenTree::Group inside '[]', found TokenTree::Punct")?,
      TokenTree::Literal(_) => Err("expected TokenTree::Group inside '[]', found TokenTree::Literal")?,
      TokenTree::Punct(_) => Err("expected TokenTree::Group inside '[]', found TokenTree::Punct")?,
    },
    None => Err("expected TokenTree::Group inside '[]', found None")?,
  }
}

pub fn parse_string_vec(tokens: &mut Tokens) -> Result<Vec<String>, String> {
  let v = parse_group(tokens, |g| {
    Ok(
      g.into_iter()
        .filter(|tok| match tok {
          TokenTree::Literal(_) => true,
          _ => false,
        }).map(|tok| trim_quote(&tok.to_string()))
        .collect(),
    )
  });
  tokens.next();
  v
}

pub fn parse_array_index(tokens: &mut Tokens) -> Result<u32, String> {
  parse_group(tokens, |g| match g.into_iter().next() {
    Some(tok) => match tok {
      TokenTree::Literal(_) => Ok(tok.to_string().parse::<u32>().map_err(|e| format!("{}", e))?),
      _ => Err("Expected TokenTree:Literal with u32")?,
    },
    None => Err("Expected TokenTree:Literal with u32")?,
  })
}
