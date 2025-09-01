#!/bin/sh
if [ -f server ]; then
    rm server
fi

cd backend && cargo build
if [ $? -ne 0 ]; then
    echo "Build failed, exiting."
    exit 1
fi
clear

cd ..
cp backend/target/debug/stream-chat-reader ./server

HOST=${1:-localhost}
PORT=${2:-8080}

./server --host $HOST --port $PORT --dev