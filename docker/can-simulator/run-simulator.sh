#!/bin/sh
set -e

# Setup the virtual CAN interface
/app/setup-vcan.sh

echo "Starting ECU simulators..."
echo ""

# Run the ECU simulator binary
# In Task 4, this will connect to vcan0 and handle CAN frames
exec /app/ecu-simulator
