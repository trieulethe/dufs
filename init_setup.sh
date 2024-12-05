#!/bin/bash

# Cập nhật hệ thống và cài đặt các gói cần thiết
echo "Updating system and installing required packages..."
sudo apt-get update
sudo apt-get install -y build-essential git zip python pandoc ffmpeg webp nginx \
                        pkg-config libssl-dev libpq-dev

# Cài đặt youtube-dl
echo "Installing youtube-dl..."
git clone https://github.com/ytdl-org/youtube-dl.git
cd youtube-dl
make
sudo cp ./youtube-dl /usr/local/bin/
sudo chmod a+rx /usr/local/bin/youtube-dl
cd ..

# Cài đặt Rust
echo "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Cài đặt mã nguồn và cấu hình
echo "Installing DUFS and setting up media directory..."
mkdir -p /opt/media
./bash.sh

echo "Setup completed successfully!"
