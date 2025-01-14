#!/bin/bash

# Create XDG runtime directory
mkdir -p /tmp/runtime-dir
chmod 700 /tmp/runtime-dir

# Start virtual X server
Xvfb :99 -screen 0 1024x768x24 &
sleep 1

# Start the server in background
/usr/local/bin/server run &
sleep 3

# Start the client
RUST_BACKTRACE=1 /usr/local/bin/client
