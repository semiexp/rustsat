use crate::*;

type Reason = i32;

const REASON_UNDET: Reason = -2;
const REASON_BRANCH: Reason = -1;
const REASON_BACKTRACK: Reason = -3;

#[derive(Debug)]
pub struct Solver {
    assignment: Vec<Value>,
    clauses: Vec<Clause>,
    queue: Vec<Literal>,
    reason: Vec<Reason>,
    queue_top: usize,
    queue_end: usize,
}

impl Solver {
    pub fn new() -> Solver {
        Solver {
            assignment: vec![Value::Undet],
            clauses: vec![],
            queue: vec![],
            reason: vec![REASON_UNDET],
            queue_top: 0,
            queue_end: 0,
        }
    }

    pub fn new_var(&mut self) -> Var {
        let id = self.assignment.len() as i32;
        self.assignment.push(Value::Undet);
        self.reason.push(REASON_UNDET);
        Var(id)
    }

    pub fn num_var(&self) -> i32 {
        self.assignment.len() as i32 - 1
    }

    pub fn add_clause(&mut self, clause: Clause) {
        self.clauses.push(clause);
    }

    pub fn assignment(&self) -> Vec<Value> {
        self.assignment.clone()
    }

    pub fn solve(&mut self) -> bool {
        'outer:
        loop {
            if self.propagate() {
                // branch
                let pivot = self.undecided_var();
                match pivot {
                    Some(var) => {
                        self.decide_checked(Literal::new(var, false), REASON_BRANCH);
                        continue 'outer;
                    }
                    None => {
                        return true;
                    }
                }
            } else {
                // inconsistent
                loop {
                    match self.queue.pop() {
                        Some(lit) => {
                            let var_id = lit.var().0 as usize;
                            let reason = self.reason[var_id];
                            if reason == REASON_BRANCH {
                                self.reason[var_id] = REASON_BACKTRACK;
                                self.assignment[var_id] = Value::Undet;
                                if !self.decide_checked(!lit, REASON_BACKTRACK) {
                                    return false;
                                }
                                continue 'outer;
                            } else {
                                self.reason[var_id] = REASON_UNDET;
                                self.assignment[var_id] = Value::Undet;
                            }
                        }
                        None => break
                    }
                }
                return false;
            }
        }
    }

    pub fn get_assignment(&self, Var(v): Var) -> Value {
        self.assignment[v as usize]
    }

    pub fn get_assignment_lit(&self, lit: Literal) -> Value {
        if lit.is_negated() {
            !self.get_assignment(lit.var())
        } else {
            self.get_assignment(lit.var())
        }
    }

    fn decide(&mut self, lit: Literal) {
        debug_assert!(self.get_assignment(lit.var()) == Value::Undet);
        self.queue.push(lit);
        self.queue_end += 1;
    }

    fn decide_checked(&mut self, lit: Literal, reason: Reason) -> bool {
        let current = self.get_assignment_lit(lit);
        match current {
            Value::True => true,
            Value::False => false,
            Value::Undet => {
                let Var(var_id) = lit.var();
                self.assignment[var_id as usize] = lit.value();
                self.reason[var_id as usize] = reason;
                self.queue.push(lit);
                self.queue_end += 1;
                true
            }
        }
    }

    fn propagate(&mut self) -> bool {
        loop {
            let mut updated = false;
            'outer:
            for i in 0..self.clauses.len() {
                let mut undet = None;
                for lit in &self.clauses[i] {
                    match self.get_assignment_lit(*lit) {
                        Value::True => continue 'outer,
                        Value::False => continue,
                        Value::Undet => match undet {
                            Some(_) => continue 'outer,
                            None => undet = Some(*lit),
                        }
                    }
                }
                match undet {
                    Some(lit) => {
                        updated = true;
                        if !self.decide_checked(lit, i as i32) {
                            return false;
                        }
                    }
                    None => return false
                }
            }
            if !updated {
                return true;
            }
        }
    }

    fn undecided_var(&self) -> Option<Var> {
        for i in 1..self.assignment.len() {
            if self.assignment[i] == Value::Undet {
                return Some(Var(i as i32));
            }
        }
        None
    }
}
