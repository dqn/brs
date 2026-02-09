# lr2oraja Rust Porting Project

## 開発の指針

- lr2oraja (Java) を Rust に完全に移植する。
- **ロジックの厳密性:** 判定計算、BMSパース、タイミング管理において、Java版と少しの狂いもないことを目指す。
- **自律的移植:** Claude は `./lr2oraja-java` のコードを分析し、依存関係の少ないコア部分から順次 `./lr2oraja-rust` へ移植する。

## テストと検証のルール

- **データ駆動検証:** 各モジュールの移植時、Claude は検証用の最小構成テストデータ（BMSファイル等）を自ら作成すること。
- **Java側の変更許可:** Java版とRust版の出力を一致させるため、Java側のコードにデバッグ用のログ出力や値の抽出メソッドを追加・変更することを許可する。
- **Golden Master Testing:** Javaの実行結果（JSON/CSV）とRustのテスト結果を比較し、一致を確認してから次の工程へ進むこと。

## 技術スタック

- **Language:** Rust
- **Engine:** Bevy (Graphics, ECS)
- **Audio:** Kira
- **Skin System:** mlua (LuaJからの移行)
- **Time Management:** 浮動小数点誤差を避けるため、可能な限り整数（マイクロ秒）で管理。
