# AGENTS

## 運用方針
- AGENTS.md は調査メモとして常に最新に保ち続ける
- 変更点や理解の更新があれば追記・修正する

## プロジェクト概要
- Rust 製の BMS (Be-Music Source) リズムゲームプレイヤー
- 仮想解像度 1920x1080 を基準に描画し、実解像度へスケーリング

## 主要構成メモ
- `src/main.rs`: ウィンドウ設定・仮想解像度レンダリング・Scene 管理
- `src/bms/*`: BMS/PMS/BMSON 読み込み、チャート構築、タイミング計算
- `src/audio/*`: キー音/BGM の読み込み・再生、時間同期スケジューリング
- `src/game/*`: 判定、ゲージ、スコア、入力、ゲーム進行
- `src/render/*`: ハイウェイ、判定表示、BGA、ゲージなどの描画部品
- `src/scene/*`: 選曲/プレイ/リザルト/設定/段位のシーン遷移
- `src/config/*`: 設定保存・キー割り当て・IR 設定
- `src/database/*`: スコア永続化 (JSON + バックアップ)
- `src/ir/*`: IR スコア送信/ランキング取得
- `src/skin/*`: JSON スキン定義
- `tests/*`: ローダー/タイミング/判定/スコアの基礎テスト

## 詳細メモ
### 譜面読み込み
- `BmsLoader::load_full` で BMS/PMS/BMSON を判定し読み込み
- BMS/PMS は `bms-rs` を使用、BMSON は JSON を直読み
- BGA イベントは bms-rs の上書き挙動を避けるためソースから再抽出
- BMSON: ticks/beat=240、ticks/measure=960、BPMイベントを考慮して時刻変換
- BMSON: `mode_hint` が popn 系なら PMS、その他は BMS7 として扱う
- BMSON: LN は CN 扱いとして LongStart/LongEnd を生成

### タイミング計算
- BPM 変更と STOP を統合イベント化し、(小節, 位置, 種別順) でソート
- 同位置では BPM 変更→STOP の順に処理
- 小節長変更を加味してミリ秒換算
- BPM 変更は `#xxx08` (定義参照) に加えて `#xxx03` (16進 BPM) も取り込み、`00` は無視

### 判定・ゲージ
- beatoraja / LR2 の判定窓切替が可能
- beatoraja(公式コード) 7KEY判定幅: PG±20/GR±60/GD±150/BD 早+220・遅-280 (ms換算)
- beatoraja のスクラッチ判定幅は通常より+10ms広い
- 判定幅は judgerank 比率 25/50/75/100/125% でスケール
- LR2 はランク別固定窓、BAD は左右対称
- beatoraja のCNリリース判定: PG±120/GR±160/GD±200/BD 早+220・遅-280
- Expand Judge で判定窓を 1.5x 拡大
- 空POOR判定: beatoraja は前後両方、LR2 は「遅押し側のみ」
- GAS (Gauge Auto Shift) に対応
- beatoraja のゲージ値(7KEY): NORMAL=+1/+1/+0.5/-3/-6/-2, HARD=+0.15/+0.12/+0.03/-5/-10/-5 等
- beatoraja のゲージ減衰(guts)は閾値ごとの段階軽減 (例: HARDは10%未満で0.4倍…50%未満で0.8倍)
- beatoraja のゲージ補正: TOTAL/LIMIT_INCREMENT/MODIFY_DAMAGE の3種
- beatoraja の LR2ゲージは専用の増減値(例: EASY系 +1.2/+1.2/+0.6 等)を使用
- brs の LR2ゲージは LR2本体の既知仕様に合わせたテーブル/補正へ更新済み
  - GROOVE/EASY: (T/n)スケール、BAD/POOR/空POOR= -4/-6/-2 (EASYは20%増減)
  - HARD: 回復0.1/0.1/0.05、BAD/POOR/空POOR=-6/-10/-2、30%以下でダメージ0.6倍
  - TOTAL/ノーツ数が低い譜面でダメージ倍率が増加するLR2式を実装
  - EX-HARDはLR2本体に存在しないため HARD と同等扱い
  - HAZARDはDEATH相当として BAD/POOR で即失敗扱い

### 入力
- BMS 7-key / PMS 9-key / DP 14-key を切替
- キーボード + ゲームパッド入力を統合判定
- ゲームパッドは gilrs を利用し、8レーン分のボタン/軸入力を割り当て可能

### 時間同期
- ゲーム内時間は Kira のクロックを基準に進行
- オーディオ再生スケジューラも同一クロックの時刻で BGM を事前スケジュール
  - 例: `sample/take_003_ogg/_take_7N.bms` の BGM(ch01) は最大 240 分割/小節 (BPM219 だと約 4.6ms 間隔) のため、フレーム更新ベースではズレやすい

### レーンオプション
- MIRROR/RANDOM/R-RANDOM はレーンマッピングで変換
- S-RANDOM/H-RANDOM は専用処理でノート配置を再構成
- S-RANDOM: 各ノートに独立した乱数レーン、LNは開始/終端で同一レーンに揃える
- H-RANDOM: 直前レーンを避ける簡易縦連回避、LN終端は開始レーンを維持
- DP の RANDOM 系は未完成で、現状は MIRROR 相当の処理

### 表示/UI
- 判定統計(FAST/SLOW含む)と BPM 表示用コンポーネントが存在
- レーンカバー(SUDDEN+/HIDDEN+/LIFT)は 0-1000 の範囲で調整

### BGA
- 画像と動画をロード (動画は ffmpeg)
- Base/Poor/Overlay の 3 レイヤー
- 再生位置に合わせて動画フレームを更新
- 動画はレイヤーごとの BGA イベント開始時刻を基準に再生する
- 動画は RGBA へスケールしてメモリバッファに保持
- 逆方向シーク検知でデコーダをリセット (100ms以上の巻き戻し)
- BMS側のBGA抽出は `#xxx04/06/07/0A` を正規表現で拾い、base36でID化

### 選曲
- ディレクトリを走査して BMS/BME/BML/PMS/BMSON を収集
- ヘッダ解析で title/artist/level を取得し、スコア用ハッシュを生成
- WalkDir で最大深さ5まで探索
- ソートは Title/Artist/Level の順で切替、レベルフィルタあり
- ScoreRepository のクリアランプを表示用に利用

### スコア保存
- JSON で保存し、破損時の退避とバックアップ復旧あり
- スコアは SHA256(譜面ファイル) をキーに保存
- ベスト判定は「クリアランプ優先 → EX スコア → 最大コンボ」の順
- 保存先は ProjectDirs を優先し、失敗時は `.brs-data` にフォールバック

### IR
- LR2IR / Mocha-IR / MinIR / Custom の4種に対応
- LR2IR 形式に合わせたオプションビット生成ロジックあり
- スコア検証は HMAC-SHA256 でスコアハッシュを生成
- 提出時刻は未来60秒以内/過去7日以内を許可
- フルコン/FAILED 等の整合性チェックを実施
- Mocha-Repository は IR API の解析/逆コンパイルを禁止しており、公開API仕様は確認できず
- Mocha-Repository の How to use では、beatoraja-mocha 版を利用して IR 設定する運用が前提
- LR2IR の検索CGIは接続タイムアウト (2026-01-17)
- MinIR / CinnamonIR の公式API/到達性は未確認 (ページ取得失敗)

## 外部仕様メモ
- BMS 仕様: https://bm98.yaneu.com/bm98/bmsformat.html
- 参考コマンド: https://hitkey.nekokan.dyndns.info/cmds.htm
- BMSON 仕様: bmson-spec / bmson-spec-fork のドキュメント

## 追加調査TODO
- IR API の最新仕様 (公開資料が見当たらないため要注意)

## 最終更新
- 2026-01-17
