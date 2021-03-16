extern crate rustsat;

use rustsat::*;

fn run_dimacs() {
    let (n_var, _, clauses) = dimacs::read_dimacs();

    let mut solver = solver::Solver::new();
    let vars = (0..n_var).map(|_| solver.new_var()).collect::<Vec<_>>();
    for clause in &clauses {
        solver.add_clause(clause.into_iter().map(|&x| if x > 0 {
            vars[(x - 1) as usize].literal()
        } else {
            !vars[(-x - 1) as usize].literal()
        }).collect());
    }

    println!("is_sat: {}", solver.solve());
    println!("{:?}", solver.assignment());
}

fn main() {
    run_dimacs();
    return;
}
