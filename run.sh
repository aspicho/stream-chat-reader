HOST=${1:-localhost}
PORT=${2:-8080}

echo "Starting server on $HOST:$PORT"
./server --host $HOST --port $PORT