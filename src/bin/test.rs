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

    println!("{}", if solver.solve() { "SAT" } else { "UNSAT" });
    println!("{:?}", solver.stats());
}

fn main() {
    let start = std::time::Instant::now();
    run_dimacs();
    let elapsed = start.elapsed().as_micros() as f64 / 1e3;
    println!("Cost: {:.3}[ms]", elapsed);
    return;
}
