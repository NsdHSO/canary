#!/bin/sh
set -e

# Setup the virtual CAN interface
/app/setup-vcan.sh

echo "CAN simulator container is running. vcan0 is ready for testing."
echo "Press Ctrl+C to stop..."

# Keep the container running
tail -f /dev/null
