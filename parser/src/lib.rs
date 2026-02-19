pub mod ast;
#[allow(warnings)]
mod bindings;
pub mod lexer;
pub mod preprocessor;

use bindings::exports::lojban::nesy::parser::Guest;
use bindings::lojban::nesy::ast_types::AstBuffer;

struct ParserComponent;

impl Guest for ParserComponent {
    fn parse_text(input: String) -> Result<AstBuffer, String> {
        // 1. Lex into classification stream
        let raw_tokens = crate::lexer::tokenize(&input);

        // 2. Resolve metalinguistics (si/sa/su/zo/zoi)
        let normalized = crate::preprocessor::preprocess(raw_tokens.into_iter(), &input);

        // 3. Structural Parse
        let ast = crate::ast::parse_tokens_to_ast(&normalized)?;

        // 4. Flatten AST to linear buffer for WIT
        Ok(flatten_to_buffer(ast))
    }
}

fn flatten_to_buffer(ast_list: Vec<crate::ast::Bridi>) -> AstBuffer {
    let mut buffer = AstBuffer {
        selbris: Vec::new(),
        sumtis: Vec::new(),
        sentences: Vec::new(),
    };

    for bridi in ast_list {
        let selbri_data = match bridi.selbri {
            crate::ast::Selbri::Root(s) => bindings::lojban::nesy::ast_types::Selbri::Root(s),
            crate::ast::Selbri::Compound(parts) => {
                bindings::lojban::nesy::ast_types::Selbri::Compound(parts)
            }
            crate::ast::Selbri::Tanru(modifier, head) => {
                let m_id = buffer.selbris.len() as u32;
                let m_data = match *modifier {
                    crate::ast::Selbri::Root(s) => {
                        bindings::lojban::nesy::ast_types::Selbri::Root(s)
                    }
                    _ => bindings::lojban::nesy::ast_types::Selbri::Root("unknown".to_string()),
                };
                buffer.selbris.push(m_data);

                let h_id = buffer.selbris.len() as u32;
                let h_data = match *head {
                    crate::ast::Selbri::Root(s) => {
                        bindings::lojban::nesy::ast_types::Selbri::Root(s)
                    }
                    _ => bindings::lojban::nesy::ast_types::Selbri::Root("unknown".to_string()),
                };
                buffer.selbris.push(h_data);

                bindings::lojban::nesy::ast_types::Selbri::Tanru((m_id, h_id))
            }
        };
        let selbri_id = buffer.selbris.len() as u32;
        buffer.selbris.push(selbri_data);

        let mut term_ids = Vec::new();
        for term in bridi.terms {
            let sumti_id = buffer.sumtis.len() as u32;
            let sumti_data = match term {
                crate::ast::Sumti::ProSumti(s) => {
                    bindings::lojban::nesy::ast_types::Sumti::ProSumti(s)
                }
                crate::ast::Sumti::Description(d_selbri) => {
                    let d_selbri_id = buffer.selbris.len() as u32;
                    let inner_selbri = match *d_selbri {
                        crate::ast::Selbri::Root(s) => {
                            bindings::lojban::nesy::ast_types::Selbri::Root(s)
                        }
                        _ => bindings::lojban::nesy::ast_types::Selbri::Root("desc".to_string()),
                    };
                    buffer.selbris.push(inner_selbri);
                    bindings::lojban::nesy::ast_types::Sumti::Description(d_selbri_id)
                }
                crate::ast::Sumti::Name(n) => bindings::lojban::nesy::ast_types::Sumti::Name(n),
                crate::ast::Sumti::QuotedLiteral(q) => {
                    bindings::lojban::nesy::ast_types::Sumti::QuotedLiteral(q)
                }
            };
            buffer.sumtis.push(sumti_data);
            term_ids.push(sumti_id);
        }

        buffer
            .sentences
            .push(bindings::lojban::nesy::ast_types::Bridi {
                relation: selbri_id,
                terms: term_ids,
            });
    }

    buffer
}

bindings::export!(ParserComponent with_types_in bindings);
