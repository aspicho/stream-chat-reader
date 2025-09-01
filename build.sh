#!/bin/sh
if [ -f server ]; then
    rm server
fi

echo "Building release version of the backend..."
cd backend && cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed, exiting."
    exit 1
fi
clear

cd ..
cp backend/target/release/stream-chat-reader ./server

echo "Build frontend..."
cd frontend && bun run build
if [ $? -ne 0 ]; then
    echo "Frontend build failed, exiting."
    exit 1
fi
cd ..
cp -r frontend/build static
clear
