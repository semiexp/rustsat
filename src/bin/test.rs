extern crate rustsat;

use rustsat::*;

fn run_dimacs() {
    let n_var;
    let clauses;

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        let stdin = std::io::stdin();
        let mut lock = stdin.lock();
        let data = dimacs::read_dimacs(&mut lock);
        n_var = data.0;
        clauses = data.2;
    } else {
        let mut fp = std::fs::File::open(&args[1]).unwrap();
        let data = dimacs::read_dimacs(&mut fp);
        n_var = data.0;
        clauses = data.2;
    }

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
