use crate::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Reason {
    Undet,
    Branch,
    Clause(usize),
}

impl Reason {
    fn is_clause(self) -> bool {
        match self {
            Reason::Clause(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Solver {
    assignment: Vec<Value>,
    clauses: Vec<Clause>,
    queue: Vec<Literal>,
    reason: Vec<Reason>,
    watcher_clauses: Vec<Vec<usize>>,
    queue_top: usize,
}

impl Solver {
    pub fn new() -> Solver {
        Solver {
            assignment: vec![],
            clauses: vec![],
            queue: vec![],
            reason: vec![],
            watcher_clauses: vec![],
            queue_top: 0,
        }
    }

    pub fn new_var(&mut self) -> Var {
        let id = self.assignment.len() as i32;
        self.assignment.push(Value::Undet);
        self.reason.push(Reason::Undet);
        self.watcher_clauses.push(vec![]);
        Var(id)
    }

    pub fn num_var(&self) -> i32 {
        self.assignment.len() as i32
    }

    pub fn add_clause(&mut self, clause: Clause) {
        let clause_id = self.clauses.len();
        for &lit in &clause {
            self.watcher_clauses[lit.var_id()].push(clause_id);
        }
        self.clauses.push(clause);
    }

    pub fn assignment(&self) -> Vec<Value> {
        self.assignment.clone()
    }

    pub fn solve(&mut self) -> bool {
        'outer:
        loop {
            if let Some(conflict) = self.propagate() {
                // inconsistent
                let learnt = self.analyze(conflict);
                loop {
                    match self.queue.pop() {
                        Some(lit) => {
                            let var_id = lit.var_id();
                            let reason = self.reason[var_id];
                            self.reason[var_id] = Reason::Undet;
                            self.assignment[var_id] = Value::Undet;
                            if reason == Reason::Branch {
                                let mut enq = None;
                                for &lit in &learnt {
                                    match self.get_assignment_lit(lit) {
                                        Value::True => panic!(),
                                        Value::False => (),
                                        Value::Undet => match enq {
                                            Some(_) => panic!(),
                                            None => enq = Some(lit),
                                        }
                                    }
                                }
                                assert!(enq.is_some());
                                self.queue_top = self.queue.len();
                                self.add_clause(learnt);
                                self.decide_checked(enq.unwrap(), Reason::Clause(self.clauses.len() - 1));
                                continue 'outer;
                            }
                        }
                        None => break
                    }
                }
                return false;
            } else {
                // branch
                let pivot = self.undecided_var();
                match pivot {
                    Some(var) => {
                        self.decide_checked(Literal::new(var, false), Reason::Branch);
                        continue 'outer;
                    }
                    None => {
                        return true;
                    }
                }
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
    }

    fn clear(&mut self, var: Var) {
        let var_id = var.0 as usize;
        self.reason[var_id] = Reason::Undet;
        self.assignment[var_id] = Value::Undet;
    }

    fn decide_checked(&mut self, lit: Literal, reason: Reason) -> bool {
        let current = self.get_assignment_lit(lit);
        match current {
            Value::True => true,
            Value::False => false,
            Value::Undet => {
                let var_id = lit.var_id();
                self.assignment[var_id] = lit.value();
                self.reason[var_id] = reason;
                self.queue.push(lit);
                true
            }
        }
    }

    fn propagate(&mut self) -> Option<Reason> {
        while self.queue_top < self.queue.len() {
            let lit = self.queue[self.queue_top];
            let var_id = lit.var_id();

            for i in 0..self.watcher_clauses[var_id].len() {
                let clause_id = self.watcher_clauses[var_id][i];
                if !self.propagate_clause(clause_id) {
                    self.queue_top = self.queue.len();
                    return Some(Reason::Clause(clause_id));
                }
            }
            self.queue_top += 1;
        }
        None
    }

    fn analyze(&mut self, mut conflict: Reason) -> Clause {
        let mut p: Option<Literal> = None;
        let mut visited = vec![false; self.num_var() as usize];
        let mut polarity = vec![Literal(0); self.num_var() as usize];
        while !self.queue.is_empty() {
            if let Some(l) = p {
                visited[l.var_id()] = false;
            }
            let mut reason = vec![];
            {
                let conflict = match conflict {
                    Reason::Clause(c) => c,
                    _ => panic!(),
                };
                for &lit in &self.clauses[conflict] {
                    if Some(lit) != p {
                        debug_assert!(p.is_none() || p.unwrap().var() != lit.var());
                        reason.push(lit);
                    }
                }
            }
            for lit in reason {
                let var_id = lit.var_id();
                visited[var_id] = true;
                polarity[var_id] = lit;
            }
            while !visited[self.queue[self.queue.len() - 1].var_id()] {
                let last = self.queue.pop();
                self.clear(last.unwrap().var());
                debug_assert!(self.reason[last.unwrap().var_id()].is_clause());
            }
            debug_assert!(!self.queue.is_empty());
            if self.reason[self.queue[self.queue.len() - 1].var_id()] == Reason::Branch {
                break;
            }
            let pb = self.queue[self.queue.len() - 1];
            let var_id = pb.var_id();
            p = Some(pb);
            debug_assert!(self.reason[var_id].is_clause());
            conflict = self.reason[var_id];
            self.clear(pb.var());
            self.queue.pop();
        }
        let mut ret = vec![];
        for i in 0..visited.len() {
            if visited[i] {
                ret.push(polarity[i]);
            }
        }
        ret
    }

    fn propagate_clause(&mut self, clause_id: usize) -> bool {
        let mut undet = None;
        for lit in &self.clauses[clause_id] {
            match self.get_assignment_lit(*lit) {
                Value::True => return true,
                Value::False => continue,
                Value::Undet => match undet {
                    Some(_) => return true,
                    None => undet = Some(*lit),
                }
            }
        }
        match undet {
            Some(lit) => self.decide_checked(lit, Reason::Clause(clause_id)),
            None => false
        }
    }

    fn undecided_var(&self) -> Option<Var> {
        for i in 0..self.assignment.len() {
            if self.assignment[i] == Value::Undet {
                return Some(Var(i as i32));
            }
        }
        None
    }
}
