# rename .cnf files in the given directory so that they are numbered by 0, 1, ...

import glob
import os
import re
import shutil
import sys


def main(dir):
    files = glob.glob(os.path.join(dir, "*.cnf"))
    agile_pattern = re.compile("bench_(\\d+)\\.smt2\\.cnf")
    entry = []
    for file in files:
        f = os.path.basename(file)
        m = agile_pattern.match(f)
        if m:
            key = int(m[1])
        else:
            key = f
        entry.append((key, file))

    entry.sort()

    for id, (_, src_path) in enumerate(entry):
        src_name = os.path.basename(src_path)
        dest_name = f"{id}.cnf"
        dest_path = os.path.join(dir, dest_name)
        shutil.move(src_path, dest_path)
        print(src_name, dest_name)


if __name__ == "__main__":
    main(sys.argv[1])
