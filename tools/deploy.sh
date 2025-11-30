#!/usr/bin/env fish
# Deployment script for RexOS to target device

set DEVICE_IP $argv[1]
set TARGET "aarch64-unknown-linux-gnu"
set REMOTE_PATH "/tmp/rexos_test"
if test -z "$DEVICE_IP"
    echo "Usage: deploy.sh <device-ip>"
    exit 1
end

echo "Building for target: $TARGET"
cross build --target $TARGET --release

if test $status -ne 0
    echo "Build failed!"
    exit 1
end

echo "Deploying to device at $DEVICE_IP..."
scp -r target/$TARGET/release/* ark@$DEVICE_IP:$REMOTE_PATH/

echo "Deployment complete!"
echo "Connect with: ssh ark@$DEVICE_IP"
