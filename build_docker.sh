#!/bin/bash
set -e

VERSION="latest"

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            shift
            shift
            ;;
        *)
            echo "未知选项: $1"
            exit 1
            ;;
    esac
done

echo "📦 开始构建版本 $VERSION ..."

# 构建镜像
sudo docker build -t touhou-music-server:$VERSION .

echo "✅ 构建完成！"
