use std::ops::RangeInclusive;

pub const START: usize = 0;

pub struct Automaton<Sym: Copy + Ord> {
    states: Vec<State<Sym>>,
    current_state: Option<usize>,
    previous_accepting: bool,
}

impl<Sym: Copy + Ord> Automaton<Sym> {
    pub fn transition(&mut self, symbol: Option<Sym>) {
        self.previous_accepting = self
            .current_state
            .map(|idx| self.states[idx].accepting)
            .unwrap_or(false);

        if let Some(state_idx) = self.current_state {
            self.current_state = self.states[state_idx].transition(symbol)
        }
    }

    pub fn is_previous_accepting(&self) -> bool {
        self.previous_accepting
    }

    pub fn is_alive(&self) -> bool {
        self.current_state.is_some()
    }

    pub fn reset(&mut self) {
        self.current_state = Some(START);
        self.previous_accepting = false;
    }
}

struct State<Sym: Copy + Ord> {
    transitions: Vec<(RangeInclusive<Sym>, usize)>,
    accepting: bool,
}

impl<Sym: Copy + Ord> State<Sym> {
    fn transition(&self, symbol: Option<Sym>) -> Option<usize> {
        if let Some(symbol) = symbol {
            for t in &self.transitions {
                let (range, next_state) = t;

                if range.contains(&symbol) {
                    return Some(*next_state);
                }
            }
        }

        None
    }

    fn new(accepting: bool) -> Self {
        Self {
            transitions: Vec::new(),
            accepting,
        }
    }
}

pub struct AutomatonBuilder<Sym: Copy + Ord> {
    states: Vec<State<Sym>>,
}

impl<Sym: Copy + Ord> AutomatonBuilder<Sym> {
    pub fn new() -> Self {
        Self {
            states: vec![State::new(false)],
        }
    }

    pub fn add_state(&mut self, accepting: bool) -> usize {
        let idx = self.states.len();
        self.states.push(State::new(accepting));
        idx
    }

    pub fn add_transition(
        &mut self,
        from: usize,
        to: usize,
        symbols: RangeInclusive<Sym>,
    ) {
        if from >= self.states.len() {
            panic!("Transition 'from' argument exceeds state count");
        }

        if to >= self.states.len() {
            panic!("Transition 'to' argument exceeds state count");
        }

        self.states[from].transitions.push((symbols, to));
    }

    pub fn build(self) -> Automaton<Sym> {
        Automaton {
            states: self.states,
            current_state: Some(START),
            previous_accepting: false,
        }
    }
}

pub fn keyword_automaton<Sym: Copy + Ord>(
    keyword: impl IntoIterator<Item = Sym>,
) -> Automaton<Sym> {
    let mut keyword = keyword.into_iter().peekable();

    let mut builder = AutomatonBuilder::new();

    while let Some(new_sym) = keyword.next() {
        let new_state_idx = builder.add_state(keyword.peek().is_none());
        builder.add_transition(
            new_state_idx - 1,
            new_state_idx,
            new_sym..=(new_sym),
        );
    }

    builder.build()
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_keyword() {
        let mut automaton = keyword_automaton("hello".chars());

        assert!(automaton.is_alive());
        assert!(!automaton.is_previous_accepting());

        automaton.transition(Some('h'));

        assert!(automaton.is_alive());
        assert!(!automaton.is_previous_accepting());

        automaton.transition(Some('e'));

        assert!(automaton.is_alive());
        assert!(!automaton.is_previous_accepting());

        automaton.transition(Some('l'));

        assert!(automaton.is_alive());
        assert!(!automaton.is_previous_accepting());

        automaton.transition(Some('l'));

        assert!(automaton.is_alive());
        assert!(!automaton.is_previous_accepting());

        automaton.transition(Some('o'));

        assert!(automaton.is_alive());
        assert!(!automaton.is_previous_accepting());

        automaton.transition(None);

        assert!(!automaton.is_alive());
        assert!(automaton.is_previous_accepting());

        automaton.reset();

        assert!(automaton.is_alive());
        assert!(!automaton.is_previous_accepting());

        automaton.transition(Some('h'));

        assert!(automaton.is_alive());
        assert!(!automaton.is_previous_accepting());

        automaton.transition(Some('h'));

        assert!(!automaton.is_alive());
        assert!(!automaton.is_previous_accepting());
    }
}
