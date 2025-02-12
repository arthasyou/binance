CREATE TABLE IF NOT EXISTS trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- 自增主键
    symbol TEXT NOT NULL,                -- 交易品种符号
    entry_price TEXT NOT NULL,           -- 入场价格
    close_price TEXT NOT NULL,             -- 止损点位
    direction TEXT NOT NULL,             -- 交易方向 ('Long' or 'Short')
    quantity TEXT NOT NULL,              -- 数量（字符串存储）
    leverage TEXT NOT NULL,              -- 杠杆倍
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 用户的唯一ID，自增
    username TEXT NOT NULL UNIQUE,         -- 用户名，唯一且不能为空
    password TEXT NOT NULL,                -- 密码，不能为空
    apikey TEXT NOT NULL,                  -- apikey，不能为空
    secret TEXT NOT NULL,                  -- secret，不能为空
    create_at IINTEGER NOT NULL DEFAULT (strftime('%s', 'now')),            -- 创建时间，存储为UNIX时间戳（整数类型）
    CONSTRAINT username_unique UNIQUE(username) -- 确保用户名唯一
);

