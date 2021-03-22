use std::io::{BufRead, Read};

pub fn read_dimacs<R: Read>(input: &mut R) -> (i32, i32, Vec<Vec<i32>>) {
    let mut handle = std::io::BufReader::new(input);

    let mut buffer = String::new();
    handle.read_line(&mut buffer).unwrap();

    let n_var;
    let n_clause;

    {
        let tokens = buffer.trim_end_matches('\n').trim_end_matches('\r').split(" ").collect::<Vec<&str>>();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], "p");
        assert_eq!(tokens[1], "cnf");

        n_var = tokens[2].parse::<i32>().unwrap();
        n_clause = tokens[3].parse::<i32>().unwrap();
    }

    let mut clauses = vec![];

    for _ in 0..n_clause {
        buffer.clear();
        handle.read_line(&mut buffer).unwrap();
        let mut tokens = buffer.trim_end_matches('\n').trim_end_matches('\r').split(" ").map(|x| x.parse::<i32>().unwrap()).collect::<Vec<_>>();
        assert_eq!(tokens.pop(), Some(0));
        clauses.push(tokens);
    }
    (n_var, n_clause, clauses)
}
