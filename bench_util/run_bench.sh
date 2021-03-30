# Usage
# ./run_bench.sh <executable> <testcase dir> <destination dir>

set -eu

SOLVER=$1
TC_DIR=$2
DEST_DIR=$3

INPUTS=`for fn in \`ls $TC_DIR/*.cnf\`; do echo ${fn##*/}; done`

mkdir $DEST_DIR

echo "$INPUTS" | xargs -P 10 -n 1 -I {} sh -c "echo {}; timeout 60s $SOLVER $TC_DIR/{} > $DEST_DIR/{}"
