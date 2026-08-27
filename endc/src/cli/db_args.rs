use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
pub struct DbArgs {
    #[command(subcommand)]
    pub action: DbAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum DbAction {
    /// Execute a SQL statement (CREATE, INSERT, UPDATE, DELETE) against a SQLite database
    Exec {
        #[arg(short, long)]
        path: String,

        #[arg(short, long)]
        sql: String,
    },

    /// Execute a SELECT query against a SQLite database and print results
    Query {
        #[arg(short, long)]
        path: String,

        #[arg(short, long)]
        sql: String,

        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// List all user tables in a SQLite database
    Tables {
        #[arg(short, long)]
        path: String,
    },
}
