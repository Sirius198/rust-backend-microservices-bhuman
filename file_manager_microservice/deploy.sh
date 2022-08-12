echo "NOTE: This script must be ran on the root of this monorepo"

aws ecr get-login-password --region us-east-1 | sudo docker login --username AWS --password-stdin 684187527326.dkr.ecr.us-east-1.amazonaws.com

sudo docker build -t file-manager-service -f file_manager_microservice/Dockerfile .

sudo docker tag file-manager-service:latest 684187527326.dkr.ecr.us-east-1.amazonaws.com/file-manager-service:latest

sudo docker push 684187527326.dkr.ecr.us-east-1.amazonaws.com/file-manager-service:latest