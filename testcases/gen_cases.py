import random
import subprocess
import sys


def emit_problem(n, clauses):
    problem = [f"p cnf {n} {len(clauses)}"]
    for clause in clauses:
        problem.append(" ".join(map(str, clause)) + " 0")
    return "\n".join(problem)


def run_minisat(n, clauses):
    problem = emit_problem(n, clauses)
    proc = subprocess.Popen("minisat",
                            stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE)
    out, _ = proc.communicate(problem.encode("ascii"))
    out = out.decode("utf-8")
    return out.split("\n")[-2] == "SATISFIABLE"


def random_clause(n):
    assert n >= 5
    size = random.randint(3, 5)
    cset = set()
    while len(cset) < size:
        cset.add(random.randint(1, n))
    res = list(cset)
    for i in range(size):
        res[i] *= random.randint(0, 1) * 2 - 1
    return res


def gen_cases(n, out):
    clauses = [random_clause(n)]
    while run_minisat(n, clauses):
        cur_len = len(clauses)
        for _ in range(cur_len):
            clauses.append(random_clause(n))

    left = len(clauses) // 2
    right = len(clauses)
    while right - left > 1:
        mid = (left + right) // 2
        if run_minisat(n, clauses[:mid]):
            left = mid
        else:
            right = mid

    with open(f"{out}_sat.txt", "w") as fp:
        print(emit_problem(n, clauses[:left]), file=fp)
    with open(f"{out}_unsat.txt", "w") as fp:
        print(emit_problem(n, clauses[:right]), file=fp)


def main():
    n = int(sys.argv[1])
    num = int(sys.argv[2])

    for i in range(num):
        print(f"Generating case #{i}", file=sys.stderr)
        gen_cases(n, f"generated/{n}_{i}")


if __name__ == "__main__":
    main()
