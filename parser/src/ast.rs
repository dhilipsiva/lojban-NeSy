use crate::lexer::LojbanToken;
use crate::preprocessor::NormalizedToken;

#[derive(Debug, PartialEq)]
pub enum Sumti {
    ProSumti(String),
    Description(Box<Selbri>),
    Name(String),
    QuotedLiteral(String),
}

#[derive(Debug, PartialEq)]
pub enum Selbri {
    Root(String),
    Compound(Vec<String>),
    Tanru(Box<Selbri>, Box<Selbri>),
}

#[derive(Debug, PartialEq)]
pub struct Bridi {
    pub selbri: Selbri,
    pub terms: Vec<Sumti>,
}

struct TokenCursor<'a> {
    tokens: &'a [NormalizedToken<'a>],
    pos: usize,
}

impl<'a> TokenCursor<'a> {
    fn peek(&self) -> Option<&'a NormalizedToken<'a>> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&'a NormalizedToken<'a>> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
}

pub fn parse_tokens_to_ast(tokens: &[NormalizedToken]) -> Result<Vec<Bridi>, String> {
    let mut cursor = TokenCursor { tokens, pos: 0 };
    let mut bridi_list = Vec::new();

    // Sentence structure: [Sumti]* [cu]? [Selbri] [Sumti]*
    let mut terms = Vec::new();
    let mut selbri = None;

    while let Some(token) = cursor.peek() {
        match token {
            NormalizedToken::Standard(LojbanToken::Cmavo, "lo") => {
                cursor.next(); // consume 'lo'
                if let Some(NormalizedToken::Standard(LojbanToken::Gismu, s)) = cursor.next() {
                    terms.push(Sumti::Description(Box::new(Selbri::Root(s.to_string()))));
                }
            }
            NormalizedToken::Standard(LojbanToken::Cmavo, s) if *s == "mi" || *s == "do" => {
                cursor.next();
                terms.push(Sumti::ProSumti(s.to_string()));
            }
            NormalizedToken::Standard(LojbanToken::Cmavo, "cu") => {
                cursor.next(); // consume 'cu' separator
            }
            NormalizedToken::Standard(LojbanToken::Gismu, s) => {
                cursor.next();
                selbri = Some(Selbri::Root(s.to_string()));
            }
            _ => {
                cursor.next();
            } // Skip unknown for MVP
        }
    }

    if let Some(s) = selbri {
        bridi_list.push(Bridi { selbri: s, terms });
        Ok(bridi_list)
    } else {
        Err("No selbri found in token stream".to_string())
    }
}
