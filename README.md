# Redpada

curl -1sLf 'https://packages.vectorized.io/nzc4ZYQK3WRGd9sy/redpanda/cfg/setup/bash.deb.sh' | sudo -E bash

sudo apt install redpanda -y

sudo systemctl start redpanda

sudo systemctl status redpanda

rpk topic create bhuman_channel --partitions 10 --replicas 1 --brokers=localhost:9092

rpk topic list

rpk topic describe bhuman_channel

rpk topic delete bhuman_channel

(production)

sudo rpk mode production

sudo rpk tune all


# CMake

sudo apt-get install build-essential libssl-dev

cd /tmp

wget https://github.com/Kitware/CMake/releases/download/v3.20.0/cmake-3.20.0.tar.gz

tar -zxvf cmake-3.20.0.tar.gz

cd cmake-3.20.0

./bootstrap

make

sudo make install

cmake --version

sudo apt install pkg-config
