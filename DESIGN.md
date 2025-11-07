# devcheck - 設計ドキュメント

## 概要

devcheckは、Rustプロジェクトの品質チェックを効率的に実行するためのCLIツールです。`cargo fmt`、`cargo clippy`、`cargo test`を並列実行し、結果を見やすく要約して表示します。

## 実装状況

**現在のバージョン: v0.1.0**

### 実装完了 ✅

**フェーズ1: コア機能**
- ✅ 基本的なfmt/clippy/test実行
- ✅ 並列実行（tokio）
- ✅ ターミナル出力（カラー対応）
- ✅ JSON出力（CI/CD対応）

**フェーズ2: 機能拡張**
- ✅ 設定ファイルサポート（`.devcheck.toml`）
- ✅ ワークスペース対応（`--workspace`）
- ✅ プログレスバー（リアルタイム表示）
- ✅ カスタムチェック追加機能
- ✅ ウォッチモード（`--watch`）
- ✅ HTML/Markdownレポート生成

### 今後の予定 🔮

**フェーズ3: 高度な機能**
- インクリメンタルチェック（変更ファイルのみ）
- キャッシュ機能
- GitHub Actions統合
- VS Code拡張連携
- プロジェクト統計情報収集

## 目的

- **効率化**: 複数のチェックを並列実行することで、開発フィードバックループを高速化
- **可視性**: チェック結果を整理された形式で表示し、問題の把握を容易に
- **自動化**: CI/CDパイプラインや開発ワークフローに簡単に組み込める
- **使いやすさ**: シンプルなCLIインターフェースで誰でも使える

## アーキテクチャ

### 全体構成

```
┌─────────────────────────────────────┐
│         CLI エントリポイント         │
│      (引数解析、設定読み込み)        │
└───────────┬─────────────────────────┘
            │
            ▼
┌─────────────────────────────────────┐
│       タスク実行エンジン             │
│    (並列実行、進捗管理)              │
└───────────┬─────────────────────────┘
            │
            ├────────┬────────┬────────┤
            ▼        ▼        ▼        ▼
    ┌───────────┐ ┌───────┐ ┌──────┐ ┌──────┐
    │ cargo fmt │ │clippy │ │ test │ │build │
    └─────┬─────┘ └───┬───┘ └──┬───┘ └──┬───┘
          │           │        │        │
          └───────────┴────────┴────────┘
                      │
                      ▼
            ┌──────────────────────┐
            │   結果集約・整形      │
            │  (サマリー生成)       │
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │    出力フォーマッター  │
            │  (ターミナル、JSON等) │
            └──────────────────────┘
```

### モジュール構成

```
devcheck/
├── src/
│   ├── main.rs              # エントリポイント
│   ├── cli.rs               # CLI引数定義・解析
│   ├── config.rs            # 設定管理
│   ├── runner/
│   │   ├── mod.rs           # タスク実行統括
│   │   ├── task.rs          # タスク定義・実行
│   │   ├── executor.rs      # 並列実行エンジン
│   │   └── output.rs        # コマンド出力キャプチャ
│   ├── checker/
│   │   ├── mod.rs           # チェッカー統括
│   │   ├── fmt.rs           # cargo fmt チェック
│   │   ├── clippy.rs        # cargo clippy チェック
│   │   ├── test.rs          # cargo test チェック
│   │   └── build.rs         # cargo build チェック
│   ├── report/
│   │   ├── mod.rs           # レポート統括
│   │   ├── summary.rs       # サマリー生成
│   │   ├── formatter.rs     # 出力フォーマット
│   │   └── types.rs         # レポート型定義
│   └── utils/
│       ├── mod.rs
│       ├── color.rs         # ターミナルカラー処理
│       └── duration.rs      # 実行時間測定
├── tests/
│   ├── integration_test.rs  # 統合テスト
│   └── fixtures/            # テスト用フィクスチャ
├── Cargo.toml
└── README.md
```

## CLI インターフェース

### 基本コマンド

```bash
# 全てのチェックを実行（デフォルト）
devcheck

# 特定のチェックのみ実行
devcheck --fmt
devcheck --clippy
devcheck --test
devcheck --build

# 複数指定も可能
devcheck --fmt --clippy

# すべて無効にして特定のものだけ有効化
devcheck --no-default --fmt
```

### オプション

```bash
# JSON形式で出力（CI/CD向け）
devcheck --format json

# 静かに実行（エラーのみ表示）
devcheck --quiet

# 詳細出力
devcheck --verbose

# カラー出力制御
devcheck --color auto|always|never

# 作業ディレクトリ指定
devcheck --manifest-path /path/to/Cargo.toml

# 並列度指定
devcheck --jobs 4

# タイムアウト設定
devcheck --timeout 300
```

### 終了コード

- `0`: 全てのチェックが成功
- `1`: 1つ以上のチェックが失敗
- `2`: コマンド実行エラー（cargo未インストール等）

## 並列実行戦略

### 実装方式

1. **tokio非同期ランタイム**を使用
   - 軽量な非同期タスク実行
   - 効率的なリソース管理

2. **tokio::process::Command**でサブプロセス実行
   - 各チェックを独立したプロセスとして実行
   - stdout/stderrを非同期にキャプチャ

3. **tokio::task::JoinSet**で並列タスク管理
   - 全タスクの完了待機
   - 個別タスクの結果収集

### 実行フロー

```rust
async fn run_checks(config: &Config) -> Result<Report> {
    let mut tasks = JoinSet::new();
    
    // 各チェックを非同期タスクとして起動
    if config.run_fmt {
        tasks.spawn(run_fmt_check());
    }
    if config.run_clippy {
        tasks.spawn(run_clippy_check());
    }
    if config.run_test {
        tasks.spawn(run_test_check());
    }
    
    // 全タスクの完了を待機
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result??);
    }
    
    // 結果を集約
    Ok(Report::from_results(results))
}
```

## 出力フォーマット

### ターミナル出力（デフォルト）

```
╭─────────────────────────────────────╮
│  devcheck - Rust Quality Checker    │
╰─────────────────────────────────────╯

Running checks in parallel...

✓ cargo fmt      [  1.2s] PASSED
✓ cargo clippy   [  3.4s] PASSED
✗ cargo test     [  5.6s] FAILED

───────────────────────────────────────

Summary:
  Total:   3 checks
  Passed:  2 checks
  Failed:  1 check
  Time:    5.6s

Failed Checks:
  ✗ cargo test
    2 tests failed:
    - tests::test_foo
    - tests::test_bar

Exit code: 1
```

### JSON出力（CI/CD向け）

```json
{
  "version": "1.0.0",
  "timestamp": "2025-11-05T12:24:58Z",
  "duration_secs": 5.6,
  "summary": {
    "total": 3,
    "passed": 2,
    "failed": 1
  },
  "checks": [
    {
      "name": "fmt",
      "status": "passed",
      "duration_secs": 1.2,
      "output": ""
    },
    {
      "name": "clippy",
      "status": "passed",
      "duration_secs": 3.4,
      "output": ""
    },
    {
      "name": "test",
      "status": "failed",
      "duration_secs": 5.6,
      "output": "running 10 tests\ntest tests::test_foo ... FAILED\n...",
      "errors": [
        "tests::test_foo",
        "tests::test_bar"
      ]
    }
  ]
}
```

## 依存クレート

### 主要依存

```toml
[dependencies]
# 非同期ランタイム
tokio = { version = "1", features = ["full"] }

# CLI引数解析
clap = { version = "4", features = ["derive"] }

# エラーハンドリング
anyhow = "1"
thiserror = "1"

# シリアライゼーション（JSON出力）
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# ターミナル出力
colored = "2"
indicatif = "0.17"

# 時刻処理
chrono = "0.4"
```

## エラーハンドリング

### エラー型設計

```rust
#[derive(Debug, thiserror::Error)]
pub enum DevCheckError {
    #[error("cargo command not found")]
    CargoNotFound,
    
    #[error("failed to execute {command}: {source}")]
    ExecutionError {
        command: String,
        #[source]
        source: std::io::Error,
    },
    
    #[error("check timed out after {timeout} seconds")]
    Timeout { timeout: u64 },
    
    #[error("invalid configuration: {0}")]
    ConfigError(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

### エラーハンドリング方針

1. **回復可能なエラー**: 警告を表示して続行
   - 一部のチェックが失敗しても他は実行
   
2. **致命的なエラー**: 即座に終了
   - cargo未インストール
   - Cargo.toml未検出
   - 重大な設定エラー

3. **タイムアウト**: 設定可能な時間でタスクを強制終了

## 将来の拡張性

### フェーズ1（初期実装）
- [x] 基本的なfmt/clippy/test実行
- [x] 並列実行
- [x] ターミナル出力
- [x] JSON出力

### フェーズ2（機能拡張）
- [x] 設定ファイルサポート（`.devcheck.toml`）
- [x] ワークスペース対応
- [x] カスタムチェック追加機能
- [x] ウォッチモード（ファイル変更検知）
- [x] HTML/Markdown レポート生成

### フェーズ3（高度な機能）
- [ ] インクリメンタルチェック（変更ファイルのみ）
- [ ] キャッシュ機能
- [ ] GitHub Actions統合
- [ ] VS Code拡張連携
- [ ] プロジェクト統計情報収集

## 実装詳細（フェーズ2）

### 設定ファイルサポート

`.devcheck.toml`ファイルでプロジェクト固有の設定を定義できます：

```toml
[devcheck]
timeout = 300
parallel = true

[devcheck.fmt]
enabled = true
args = []

[devcheck.clippy]
enabled = true
args = ["--", "-D", "warnings"]

[devcheck.test]
enabled = true
args = []

[devcheck.build]
enabled = false
args = []
```

### ワークスペース対応

`--workspace`フラグを使用すると、すべてのワークスペースメンバーに対してチェックを実行します：

```bash
devcheck --workspace
devcheck --workspace --fmt --clippy
```

各チェッカーは自動的に`--workspace`または`--all`フラグをcargoコマンドに追加します。

### カスタムチェック

任意のコマンドをカスタムチェックとして追加可能：

```toml
[devcheck.custom.security-audit]
command = "cargo"
enabled = true
args = ["audit"]

[devcheck.custom.doc-coverage]
command = "cargo"
enabled = true
args = ["doc", "--no-deps"]

[devcheck.custom.custom-script]
command = "./scripts/check.sh"
enabled = true
args = []
```

`CustomChecker`実装により、任意のコマンドを並列実行フレームワークに統合できます。

### ウォッチモード

`--watch`フラグでファイル変更を監視し、自動的にチェックを再実行：

```bash
devcheck --watch
devcheck -w
```

**機能：**
- `.rs`と`.toml`ファイルの変更を監視
- 再帰的にディレクトリを監視
- 1秒のデバウンス機能で頻繁な再実行を防止
- 変更検知時に視覚的なフィードバック

**実装：**
- `notify`クレートを使用したファイルシステム監視
- 非同期ループでのイベント処理
- 変更検知時に新しい`Config`インスタンスで再実行

### HTML/Markdownレポート

複数のフォーマットでレポートを生成：

**HTMLレポート (`--format html`):**
- レスポンシブデザイン
- カラーコード化されたステータスバッジ
- グリッドレイアウトでの統計表示
- プロフェッショナルなスタイリング
- ブラウザで表示可能

**Markdownレポート (`--format markdown`):**
- GitHub互換のMarkdown
- 絵文字ステータスインジケーター（✅/❌）
- `<details>`タグを使用した折りたたみ可能な出力
- ドキュメントに埋め込み可能

**使用例：**
```bash
devcheck --format html > report.html
devcheck --format markdown > REPORT.md
devcheck --format json > report.json
```

### プログレスバー

実行中のチェックをリアルタイムで表示：

```
⠁ Running fmt...
⠁ Running clippy...
⠁ Running test...
```

- `indicatif`クレートを使用
- ターミナル出力モードでのみ表示
- 各チェックの進行状況を視覚化
- 完了時に自動的にクリア

### 拡張ポイント

1. **チェッカープラグイン**
   ```rust
   pub trait Checker: Send + Sync {
       fn name(&self) -> &str;
       async fn run(&self, config: &Config) -> Result<CheckResult>;
   }
   ```

2. **フォーマッタープラグイン**
   ```rust
   pub trait Formatter: Send + Sync {
       fn format(&self, report: &Report) -> Result<String>;
   }
   ```

3. **設定拡張**
   ```toml
   [devcheck]
   timeout = 300
   parallel = true
   
   [devcheck.fmt]
   enabled = true
   args = ["--check"]
   
   [devcheck.clippy]
   enabled = true
   args = ["--", "-D", "warnings"]
   
   [devcheck.test]
   enabled = true
   args = ["--all-features"]
   
   [devcheck.custom.security-audit]
   command = "cargo audit"
   enabled = true
   ```

## 実装優先順位

### Phase 1: MVP（最小実装）✅
1. ✓ プロジェクト構造作成
2. ✓ CLI基本実装（clap）
3. ✓ 単一チェック実行（fmt/clippy/test）
4. ✓ 基本的な出力
5. ✓ エラーハンドリング

### Phase 2: 並列実行 ✅
1. ✓ tokio統合
2. ✓ 並列タスク実行
3. ✓ 結果集約

### Phase 3: 出力改善 ✅
1. ✓ カラフルなターミナル出力
2. ✓ プログレスバー
3. ✓ JSON出力

### Phase 4: 機能拡張 ✅
1. ✓ 設定ファイルサポート
2. ✓ ワークスペース対応
3. ✓ カスタムチェック
4. ✓ ウォッチモード
5. ✓ HTML/Markdownレポート
6. ✓ ドキュメント整備

### Phase 5: 高度な機能（今後の予定）
1. インクリメンタルチェック
2. キャッシュ機能
3. GitHub Actions統合
4. テスト追加
5. CI/CD設定

## テスト戦略

### 単体テスト
- 各モジュールの単体テスト
- モックを使用したチェッカーテスト
- フォーマッターのスナップショットテスト

### 統合テスト
- 実際のRustプロジェクトでの実行テスト
- 様々な失敗パターンのテスト
- 並列実行の正確性テスト

### E2Eテスト
- CLIコマンドの実行テスト
- 終了コードの検証
- 出力フォーマットの検証

## パフォーマンス考慮事項

1. **並列実行**: CPUコア数に応じた最適な並列度
2. **メモリ効率**: ストリーミングで出力を処理
3. **起動時間**: 最小限の依存で高速起動
4. **キャッシュ**: 将来的にはcargo自体のキャッシュ機能を活用

## セキュリティ考慮事項

1. **コマンドインジェクション**: 外部入力を適切にサニタイズ
2. **パス検証**: manifest-pathなどの検証
3. **権限**: 最小限の権限で実行
4. **依存クレート**: 定期的なセキュリティ監査

## まとめ

devcheckは、Rust開発者の生産性を向上させるシンプルで強力なツールを目指します。並列実行による高速化と見やすい出力により、開発フローに自然に組み込める設計となっています。

初期実装では基本機能に集中し、段階的に高度な機能を追加していく方針です。拡張性を考慮した設計により、将来的なカスタマイズや機能追加が容易になります。
