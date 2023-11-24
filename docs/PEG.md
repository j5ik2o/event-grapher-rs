```peg
# 文書は複数の要素とそれらの関係から成る
Document = (Element / Relationship)+

# 要素
Element = Actor / Event / Command / Aggregate / Policy / ReadModel

# 関係は線や矢印で表現される
Relationship = Line / Arrow

# アクター
Actor = 'ac:' WS Name

# イベント
Event = 'e:' WS Name

# コマンド
Command = 'c:' WS Name

# 集約
Aggregate = 'a:' WS Name

# ポリシー
Policy = 'p:' WS Name

# リードモデル
ReadModel = 'r:' WS Name

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
