# Copilot Instructions for `bevy_2048`

## 注意事項

- 必ず日本語で回答してください
- コード生成、セットアップや設定手順、またはライブラリやAPIドキュメントが必要な場合は常にcontext7を使用してください
- Web検索をする際は、tavilyを使用してください
- コード生成の際は、最後にかならずテスト、lint、フォーマットをしてください

## ビルド、実行、テストのコマンド

- ローカルでゲームを実行:
  - `bevy run`
- テスト:
  - `cargo test`
- リント:
  - `cargo clippy && bevy lint`
- フォーマット:
  - `cargo fmt`

## アーキテクチャ概要

- コアとなるゲームロジックは現在 `src/game.rs` にあります:
  - `Board` は中心となるデータモデル: 4x4 グリッドを `[Option<NonZero<u8>>; 16]` で保持
  - タイル値は生の数値ではなく指数で保持（`Some(1)` => 2、`Some(2)` => 4 など）

## リポジトリの主要な規約

- フィーチャーフラグは開発フローの一部です:
  - ネイティブ開発では `default = ["dev_native"]` を使用
  - Web 向けのチェック/ビルドでは `dev` を使用（`--no-default-features --features dev`）
- Lint 規約は明示されています:
  - `clippy.toml` は `children![]` マクロの波かっこスタイルを強制
  - `Cargo.toml` では Bevy ECS の system/query パターンに合わせて `too_many_arguments` と `type_complexity` を許可
- Windows ビルドでは `.cargo/config.toml` 経由で `rust-lld.exe` を使用します。リンカや rustflags の挙動を変更する際はこの点に注意してください。
