use crate::dfa;
use std::cell::Cell;

pub trait TokenKind: Copy + Eq {
    fn unknown() -> Self;
    fn has_text(&self) -> bool;
}

pub struct Lexer<Sym: Copy + Ord, K: TokenKind> {
    automata: Vec<(dfa::Automaton<Sym>, K)>,
    active_automata: Vec<usize>,
    token_text: Cell<Vec<Sym>>,
}

impl<Sym: Copy + Ord, K: TokenKind> Lexer<Sym, K> {
    pub fn new(automata: Vec<(dfa::Automaton<Sym>, K)>) -> Self {
        let active_automata = (0..automata.len()).collect();

        Self {
            automata,
            active_automata,
            token_text: vec![].into(),
        }
    }

    fn step(&mut self, symbol: Option<Sym>) -> Option<Token<Sym, K>> {
        self.active_automata
            .retain(|idx| self.automata[*idx].0.is_alive());

        let mut any_alive = false;

        for idx in &self.active_automata {
            let (ref mut automaton, _) = &mut self.automata[*idx];
            automaton.transition(symbol);
            any_alive = any_alive || automaton.is_alive();
        }

        let mut token = None;

        if !any_alive {
            for idx in &self.active_automata {
                let (ref automaton, token_kind) = &self.automata[*idx];

                if automaton.is_previous_accepting() {
                    token = Some(Token::new(
                        *token_kind,
                        self.token_text.replace(vec![]),
                    ));

                    break;
                }
            }

            if token.is_none() {
                token = Some(Token::new(
                    K::unknown(),
                    self.token_text.replace(vec![]),
                ))
            }

            self.reset_automata();
            
            for (automaton, _) in &mut self.automata {
                automaton.transition(symbol);
            }
        }

        if let Some(sym) = symbol {
            self.token_text.get_mut().push(sym);
        }

        token
    }

    fn reset_automata(&mut self) {
        self.automata
            .iter_mut()
            .for_each(|(automaton, _)| automaton.reset());
        self.active_automata = (0..self.automata.len()).collect();
    }

    pub fn lex(
        mut self,
        symbols: impl Iterator<Item = Option<Sym>>,
    ) -> impl Iterator<Item = Token<Sym, K>> {
        symbols.flat_map(move |symbol| self.step(symbol))
    }
}

pub struct Token<Sym: Copy + Ord, K: TokenKind> {
    kind: K,
    text: Option<Vec<Sym>>,
}

impl<Sym: Copy + Ord, K: TokenKind> Token<Sym, K> {
    fn new(kind: K, text: Vec<Sym>) -> Self {
        Token {
            kind,
            text: if kind.has_text() { Some(text) } else { None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dfa;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum TestLexerTokenKind {
        While,
        If,
        Ident,
        Paren,
        Unknown,
    }

    impl TokenKind for TestLexerTokenKind {
        fn unknown() -> Self {
            Self::Unknown
        }

        fn has_text(&self) -> bool {
            self == &Self::Ident
        }
    }

    fn ident_dfa() -> dfa::Automaton<u8> {
        let lowercase = b'a'..=b'z';
        let uppercase = b'A'..=b'Z';
        let digit = b'0'..=b'9';
        let underscore = b'_'..=b'_';

        let mut ident_builder = dfa::AutomatonBuilder::<u8>::new();
        let rest = ident_builder.add_state(true);
        ident_builder.add_transition(dfa::START, rest, lowercase.clone());
        ident_builder.add_transition(dfa::START, rest, uppercase.clone());
        ident_builder.add_transition(dfa::START, rest, underscore.clone());
        ident_builder.add_transition(rest, rest, lowercase);
        ident_builder.add_transition(rest, rest, uppercase);
        ident_builder.add_transition(rest, rest, underscore);
        ident_builder.add_transition(rest, rest, digit);
        ident_builder.build()
    }

    #[test]
    fn get_tokens() {
        let while_dfa = dfa::keyword_automaton(*b"while");
        let if_dfa = dfa::keyword_automaton(*b"if");
        let paren_dfa = dfa::keyword_automaton(*b"(");

        let lexer = Lexer::new(vec![
            (while_dfa, TestLexerTokenKind::While),
            (if_dfa, TestLexerTokenKind::If),
            (paren_dfa, TestLexerTokenKind::Paren),
            (ident_dfa(), TestLexerTokenKind::Ident),
        ]);

        let byte_iter = "if  while _neat1(cool 123f"
            .bytes()
            .map(|b| Some(b))
            .chain(Some(None));

        let mut token_iter = lexer.lex(byte_iter);
        let get_kind = |t: Token<_, _>| t.kind;

        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::If)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Unknown)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Unknown)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::While)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Unknown)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Ident)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Paren)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Ident)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Unknown)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Unknown)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Unknown)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Unknown)
        );
        assert_eq!(
            token_iter.next().map(get_kind),
            Some(TestLexerTokenKind::Ident)
        );
        assert_eq!(token_iter.next().map(get_kind), None);
    }

    #[test]
    fn get_ident() {
        let lexer = Lexer::new(vec![
            (ident_dfa(), TestLexerTokenKind::Ident)
        ]);

        let byte_iter = "_hello123"
            .bytes()
            .map(|b| Some(b))
            .chain(Some(None));

        let mut token_iter = lexer.lex(byte_iter);
        let token = token_iter.next().unwrap();
        
        assert_eq!(token.kind, TestLexerTokenKind::Ident);
        assert_eq!(token.text, Some("_hello123".bytes().collect()));
    }
}
