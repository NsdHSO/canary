#!/bin/bash
modprobe vcan
ip link add dev vcan0 type vcan
ip link set up vcan0
echo "✅ Virtual CAN interface vcan0 created"
candump vcan0 &  # Monitor CAN traffic
