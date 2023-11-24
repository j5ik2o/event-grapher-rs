```peg
# 文書は複数の要素とそれらの関係から成る
Document = (Element / Relationship)+

# 要素
Element = Actor / Event / Command / Aggregate / Policy / View

# 関係は線や矢印で表現される
Relationship = Line / Arrow

# アクター
Actor = 'actor' WS Name

# イベント
Event = 'event' WS Name

# コマンド
Command = 'command' WS Name

# 集約
Aggregate = 'aggregate' WS Name

# ポリシー
Policy = 'policy' WS Name

# リードモデル
ReadModel = 'view' WS Name

# 線の定義
Line = Name WS '--' WS Name (WS ':' WS Caption)?

# 矢印の定義
Arrow = Name WS '->' WS Name (WS ':' WS Caption)?

# 名前の定義
Name = '"' (!'"' Char)* '"'

# キャプションの定義
Caption = '"' (!'"' Char)* '"'

# 文字の定義
Char = .

# 空白文字
WS = [ \t\n\r]*
```
