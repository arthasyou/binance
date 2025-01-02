CREATE TABLE trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- 自增主键
    symbol TEXT NOT NULL,                -- 交易品种符号
    entry_price TEXT NOT NULL,           -- 入场价格
    close_price TEXT NOT NULL,             -- 止损点位
    direction TEXT NOT NULL,             -- 交易方向 ('Long' or 'Short')
    quantity TEXT NOT NULL,              -- 数量（字符串存储）
    leverage TEXT NOT NULL,              -- 杠杆倍
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
