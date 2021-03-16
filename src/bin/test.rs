extern crate rustsat;

use rustsat::*;

fn main() {
    let mut solver = solver::Solver::new();
    let x = solver.new_var().literal();
    let y = solver.new_var().literal();
    let z = solver.new_var().literal();

    solver.add_clause(vec![x, y]);
    solver.add_clause(vec![x, !y]);
    solver.add_clause(vec![!x, !y]);
    solver.add_clause(vec![y, !z]);
    solver.add_clause(vec![!x, z]);

    println!("{}", solver.solve());
    println!("{:?}", solver.assignment());
}
