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
    var_activity: Activity,
    trail_boundary: Vec<usize>,
    level: Vec<i32>,
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
            var_activity: Activity::new(1.0f64 / 0.95f64),
            trail_boundary: vec![],
            level: vec![],
        }
    }

    pub fn new_var(&mut self) -> Var {
        let id = self.assignment.len() as i32;
        self.assignment.push(Value::Undet);
        self.reason.push(Reason::Undet);

        // each for positive/negative literals
        self.watcher_clauses.push(vec![]);
        self.watcher_clauses.push(vec![]);

        self.var_activity.add_entry();
        self.level.push(-1);
        Var(id)
    }

    pub fn num_var(&self) -> i32 {
        self.assignment.len() as i32
    }

    pub fn add_clause(&mut self, clause: Clause) {
        let clause_id = self.clauses.len();
        for &lit in &clause {
            self.watcher_clauses[(!lit).watch_id()].push(clause_id);
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
                    if self.queue.is_empty() {
                        break;
                    }
                    let lit = self.queue[self.queue.len() - 1];
                    let var_id = lit.var_id();
                    let reason = self.reason[var_id];
                    self.pop_queue();
                    if reason == Reason::Branch {
                        let mut enq = None;
                        let mut max_level = 1;
                        for &lit in &learnt {
                            match self.get_assignment_lit(lit) {
                                Value::True => panic!(),
                                Value::False => {
                                    max_level = max_level.max(self.level[lit.var_id()]);
                                },
                                Value::Undet => match enq {
                                    Some(_) => panic!(),
                                    None => enq = Some(lit),
                                }
                            }
                            self.var_activity.bump(lit.var_id());
                        }
                        assert!(enq.is_some());
                        debug_assert!(self.queue.len() == self.trail_boundary[self.trail_boundary.len() - 1]);
                        while self.trail_boundary.len() as i32 > max_level {
                            self.pop_level();
                        }
                        self.queue_top = self.queue.len();
                        self.add_clause(learnt);
                        self.decide_checked(enq.unwrap(), Reason::Clause(self.clauses.len() - 1));
                        self.var_activity.decay();
                        continue 'outer;
                    }
                }
                return false;
            } else {
                // branch
                let pivot = self.var_activity.find_undecided(self);
                match pivot {
                    Some(var) => {
                        self.assume(Literal::new(var, false));
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

    fn assume(&mut self, lit: Literal) {
        self.trail_boundary.push(self.queue.len());
        assert!(self.decide_checked(lit, Reason::Branch));
    }

    fn clear(&mut self, var: Var) {
        let var_id = var.0 as usize;
        self.reason[var_id] = Reason::Undet;
        self.assignment[var_id] = Value::Undet;
        self.level[var_id] = -1;
    }

    fn pop_queue(&mut self) {
        // Popping beyond the trail boundary is prohibited
        assert!(self.trail_boundary[self.trail_boundary.len() - 1] != self.queue.len());

        let var = self.queue.pop().unwrap().var();
        self.clear(var);
    }

    fn pop_level(&mut self) {
        debug_assert!(self.trail_boundary[self.trail_boundary.len() - 1] <= self.queue.len());
        for _ in 0..(self.queue.len() - self.trail_boundary[self.trail_boundary.len() - 1]) {
            self.pop_queue();
        }
        self.trail_boundary.pop();
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
                self.level[var_id] = self.trail_boundary.len() as i32;
                self.queue.push(lit);
                true
            }
        }
    }

    fn propagate(&mut self) -> Option<Reason> {
        while self.queue_top < self.queue.len() {
            let lit = self.queue[self.queue_top];

            let watch_id = lit.watch_id();
            for i in 0..self.watcher_clauses[watch_id].len() {
                let clause_id = self.watcher_clauses[watch_id][i];
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
        let mut counter = 0;
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
                if !visited[var_id] && self.level[var_id] == self.trail_boundary.len() as i32 {
                    counter += 1;
                }
                visited[var_id] = true;
                polarity[var_id] = lit;
            }
            while !visited[self.queue[self.queue.len() - 1].var_id()] {
                self.pop_queue();
            }
            debug_assert!(!self.queue.is_empty());
            counter -= 1;
            if counter == 0 {
                break;
            }
            debug_assert!(self.reason[self.queue[self.queue.len() - 1].var_id()] != Reason::Branch);
            let pb = self.queue[self.queue.len() - 1];
            let var_id = pb.var_id();
            p = Some(pb);
            debug_assert!(self.reason[var_id].is_clause());
            conflict = self.reason[var_id];
            self.pop_queue();
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
            Some(lit) => {
                self.decide_checked(lit, Reason::Clause(clause_id))
            }
            None => false
        }
    }
}

const ACTIVITY_THRESHOLD: f64 = 1e100;

#[derive(Debug)]
struct Activity {
    activity: Vec<f64>,
    var_inc: f64,
    var_decay: f64,
}

impl Activity {
    fn new(var_decay: f64) -> Activity {
        Activity {
            activity: vec![],
            var_inc: 1.0,
            var_decay,
        }
    }

    fn add_entry(&mut self) {
        self.activity.push(0.0f64);
    }
    fn bump(&mut self, i: usize) {
        self.activity[i] += self.var_inc;
        if self.activity[i] > ACTIVITY_THRESHOLD {
            self.rescale();
        }
    }
    fn decay(&mut self) {
        self.var_inc *= self.var_decay;
    }

    fn rescale(&mut self) {
        for a in &mut self.activity {
            *a *= 1.0 / ACTIVITY_THRESHOLD;
        }
        self.var_inc /= ACTIVITY_THRESHOLD;
    }

    fn find_undecided(&self, solver: &Solver) -> Option<Var> {
        let mut best: Option<usize> = None;
        for i in 0..self.activity.len() {
            if solver.assignment[i] == Value::Undet {
                if match best {
                    Some(prev) => self.activity[prev] < self.activity[i],
                    None => true,
                } {
                    best = Some(i);
                }
            }
        }
        best.map(|i| Var(i as i32))
    }
}
