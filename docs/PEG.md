```peg
# 文書は複数の要素とそれらの関係から成る
Document = (Element / Relationship)+

# 要素
Element = User / Event / Command / Aggregate / Policy / ReadModel

# 関係は線や矢印で表現される
Relationship = Line / Arrow

# 利用者
User = 'u:' WS Name (WS ':' WS Caption)?

# コマンド
Command = 'c:' WS Name (WS ':' WS Caption)?

# イベント
Event = 'e:' WS Name (WS ':' WS Caption)?

# 集約
Aggregate = 'a:' WS Name　(WS ':' WS Caption)?

# ポリシー
Policy = 'p:' WS Name　(WS ':' WS Caption)?

# リードモデル
ReadModel = 'r:' WS Name　(WS ':' WS Caption)?

# 線の定義
Line = Name WS '--' WS Name (WS ':' WS Caption)?

# 矢印の定義
Arrow = Name WS '->' WS Name (WS ':' WS Caption)?

# 名前の定義
Name = (!'"' Char)* 

# キャプションの定義
Caption = '"' (!'"' Char)* '"'

# 文字の定義
Char = .

# 空白文字
WS = [ \t\n\r]*
```
