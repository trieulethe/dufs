#!/bin/bash

# Variables
APP_NAME="uploadsystem"
APP_DIRECTORY=$(pwd)
EXECUTABLE="target/release/upload"
USERNAME="root"

# Change to the app directory
cd $APP_DIRECTORY

# Run pre-setup commands
cargo build --release

# Create systemd service file
cat <<EOF > /etc/systemd/system/$APP_NAME.tld.service
[Unit]
Description=Upload System
[Service]
User=$USERNAME
Group=$USERNAME
WorkingDirectory=$APP_DIRECTORY
ExecStart=$APP_DIRECTORY/$EXECUTABLE  /opt/media/ --allow-all
[Install]
WantedBy=multi-user.target
EOF

# Reload systemd to read the new service file
systemctl daemon-reload

# Start the Rust application service
systemctl start $APP_NAME.tld.service

# Enable the service to start on boot
systemctl enable $APP_NAME.tld.service

echo "Setup complete. Your Rust application is now running as a systemd service."