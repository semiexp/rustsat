/*
An implementation of SAT solver, based on "An Extensible SAT-solver" (Niklas Een, Niklas Sörensson, SAT 2003)
*/

use crate::*;
use std::ops::Index;

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

#[derive(Debug, Clone, Copy)]
pub struct SolverStats {
    pub n_restart: u64,
    pub n_conflict: u64,
    pub n_propagation: u64,
}

impl SolverStats {
    fn new() -> SolverStats {
        SolverStats {
            n_restart: 0,
            n_conflict: 0,
            n_propagation: 0,
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
    cla_activity: Activity,
    trail_boundary: Vec<usize>,
    level: Vec<i32>,
    cla_erased: Vec<bool>,
    learnt: Vec<usize>,
    stats: SolverStats,
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
            cla_activity: Activity::new(1.0f64 / 0.999f64),
            trail_boundary: vec![],
            level: vec![],
            cla_erased: vec![],
            learnt: vec![],
            stats: SolverStats::new(),
        }
    }

    pub fn stats(&self) -> SolverStats {
        self.stats
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

    pub fn add_clause(&mut self, mut clause: Clause) -> bool {
        let clause_id = self.clauses.len();
        if clause.len() == 0 {
            return false;
        }
        if clause.len() == 1 {
            return self.decide_checked(clause[0], Reason::Branch /* TODO */);
        }
        // TODO: choose better watcher for learnt clauses
        self.watcher_clauses[(!clause[0]).watch_id()].push(clause_id);
        if clause.len() >= 2 {
            let mut max_i = 1;
            let mut max_level = self.level[clause[1].var_id()];
            for i in 2..clause.len() {
                if max_level < self.level[clause[i].var_id()] {
                    max_i = i;
                    max_level = self.level[clause[i].var_id()];
                }
            }
            if max_i != 1 {
                clause.swap(1, max_i);
            }
            self.watcher_clauses[(!clause[1]).watch_id()].push(clause_id);
        }
        self.clauses.push(clause);
        self.cla_activity.add_entry();
        self.cla_activity.bump(clause_id);
        self.cla_erased.push(false);
        true
    }

    pub fn assignment(&self) -> Vec<Value> {
        self.assignment.clone()
    }

    pub fn solve(&mut self) -> bool {
        let mut confl_threshold = 100u64;
        let mut learnt_threshold = self.clauses.len() as u64 / 3;

        loop {
            self.stats.n_restart += 1;
            self.queue_top = 0;  // TODO: should be verified
            match self.search(confl_threshold, learnt_threshold) {
                Value::True => return true,
                Value::False => return false,
                Value::Undet => (),
            }
            confl_threshold = (confl_threshold as f64 * 1.5f64) as u64;
            learnt_threshold = (learnt_threshold as f64 * 1.1f64) as u64;
        }
    }

    pub fn search(&mut self, confl_threshold: u64, learnt_threshold: u64) -> Value {
        let mut n_confl = 0;
        'outer:
        loop {
            if let Some(conflict) = self.propagate() {
                if self.trail_boundary.len() == 0 {
                    return Value::False;
                }

                n_confl += 1;
                self.stats.n_conflict += 1;

                // inconsistent
                let learnt = self.analyze(conflict);
                self.pop_level();
                let mut enq = None;
                let mut max_level = 0;
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
                }
                assert!(enq.is_some());
                while self.trail_boundary.len() as i32 > max_level {
                    self.pop_level();
                }
                self.queue_top = self.queue.len();
                if learnt.len() >= 2 {
                    self.add_clause(learnt);
                    self.learnt.push(self.clauses.len() - 1);
                    self.decide_checked(enq.unwrap(), Reason::Clause(self.clauses.len() - 1));
                } else {
                    assert!(self.add_clause(learnt));
                }
                self.var_activity.decay();
                self.cla_activity.decay();
            } else {
                if n_confl > confl_threshold {
                    while self.trail_boundary.len() > 0 {
                        self.pop_level();
                    }
                    return Value::Undet;
                }
                if self.learnt.len() as i64 - self.queue.len() as i64 > learnt_threshold as i64 {
                    self.reduce_db();
                }

                // branch
                let pivot = self.var_activity.find_undecided(self);
                match pivot {
                    Some(var) => {
                        self.assume(Literal::new(var, false));
                        continue 'outer;
                    }
                    None => {
                        return Value::True;
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
            self.stats.n_propagation += 1;

            let watch_id = lit.watch_id();
            let mut watchers = vec![];
            std::mem::swap(&mut self.watcher_clauses[watch_id], &mut watchers);

            for i in 0..watchers.len() {
                let clause_id = watchers[i];
                if self.cla_erased[clause_id] {
                    continue;
                }
                if !self.propagate_clause(clause_id, lit) {
                    // reinsert remaining watchers
                    for j in (i + 1)..watchers.len() {
                        let w = watchers[j];
                        if !self.cla_erased[w] {
                            self.watcher_clauses[watch_id].push(w);
                        }
                    }
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
        let mut counter = 0;

        let mut ret = vec![Literal(0)];
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
                self.cla_activity.bump(conflict);
                for &lit in &self.clauses[conflict] {
                    if Some(lit) != p {
                        debug_assert!(p.is_none() || p.unwrap().var() != lit.var());
                        reason.push(lit);
                    }
                }
            }
            for lit in reason {
                let var_id = lit.var_id();
                if !visited[var_id] {
                    self.var_activity.bump(lit.var_id());
                    if self.level[var_id] == self.trail_boundary.len() as i32 {
                        counter += 1;
                    } else {
                        ret.push(lit);
                    }
                    visited[var_id] = true;
                }
            }
            while !visited[self.queue[self.queue.len() - 1].var_id()] {
                self.pop_queue();
            }
            debug_assert!(!self.queue.is_empty());
            let pb = self.queue[self.queue.len() - 1];
            let var_id = pb.var_id();
            p = Some(pb);
            counter -= 1;
            if counter == 0 {
                ret[0] = !p.unwrap();
                break;
            }
            debug_assert!(self.reason[self.queue[self.queue.len() - 1].var_id()] != Reason::Branch);
            debug_assert!(self.reason[var_id].is_clause());
            conflict = self.reason[var_id];
            self.pop_queue();
        }
        ret
    }

    fn propagate_clause(&mut self, clause_id: usize, p: Literal) -> bool {
        if self.clauses[clause_id][0] == !p {
            self.clauses[clause_id].swap(0, 1);
        }
        debug_assert!(self.clauses[clause_id][1] == !p);

        if self.get_assignment_lit(self.clauses[clause_id][0]) == Value::True {
            self.watcher_clauses[p.watch_id()].push(clause_id);
            return true;
        }

        for i in 2..self.clauses[clause_id].len() {
            if self.get_assignment_lit(self.clauses[clause_id][i]) != Value::False {
                self.clauses[clause_id].swap(1, i);
                self.watcher_clauses[(!self.clauses[clause_id][1]).watch_id()].push(clause_id);
                return true;
            }
        }

        self.watcher_clauses[p.watch_id()].push(clause_id);
        return self.decide_checked(self.clauses[clause_id][0], Reason::Clause(clause_id))
    }

    fn reduce_db(&mut self) {
        {
            let cla_activity = &self.cla_activity;
            self.learnt.sort_by(|&x, &y| {
                cla_activity[x].partial_cmp(&cla_activity[y]).unwrap()
            });
        }
        let mut learnt_nxt = vec![];
        let threshold = self.cla_activity.var_inc / self.learnt.len() as f64;
        for i in 0..self.learnt.len() {
            let c = self.learnt[i];
            let is_locked = (&self.clauses[c]).into_iter().any(|lit| self.reason[lit.var_id()] == Reason::Clause(c));

            if !is_locked && (i < self.learnt.len() / 2 || self.cla_activity[c] < threshold) {
                self.cla_erased[c] = true;
            } else {
                learnt_nxt.push(c);
            }
        }
        self.learnt = learnt_nxt;
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

impl Index<usize> for Activity {
    type Output = f64;

    fn index(&self, i: usize) -> &f64 {
        &self.activity[i]
    }
}
