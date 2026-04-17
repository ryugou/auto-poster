pub mod collect;
pub mod generate;
pub mod migrate;
pub mod operate;
pub mod seed;
pub mod serve;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// SQLite マイグレーションを適用する
    Migrate,
    /// YAML 設定を DB に同期する
    Seed,
    /// 情報収集を実行する
    Collect {
        #[arg(long)]
        account: Option<String>,
    },
    /// 投稿ドラフトを生成する
    Generate {
        #[arg(long)]
        account: Option<String>,
    },
    /// レビュー TUI を起動する
    Operate,
    /// ダッシュボード API + UI サーバを起動する
    Serve {
        #[arg(long, default_value = "0.0.0.0:3000")]
        addr: String,
    },
}
