#!/bin/sh
set -e

echo "Setting up virtual CAN interface (vcan0)..."

# Load the vcan kernel module
modprobe vcan

# Create virtual CAN interface
ip link add dev vcan0 type vcan

# Bring up the interface
ip link set up vcan0

echo "vcan0 interface created and activated"
ip link show vcan0
