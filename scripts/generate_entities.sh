#!/bin/bash

# 切换到脚本所在目录的父目录
cd "$(dirname "$0")/.."

# 设置数据库连接 URL 和目标目录
DATABASE_URL="sqlite://fin.db"
OUTPUT_DIR="./src/orm"

# 运行 sea-orm-cli generate entity 命令
sea-orm-cli generate entity --database-url "$DATABASE_URL" -o "$OUTPUT_DIR"

# 检查命令是否成功执行
if [ $? -eq 0 ]; then
  echo "实体模型文件生成成功！"
else
  echo "生成失败，请检查错误。"
fi
