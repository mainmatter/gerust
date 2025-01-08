#![allow(missing_docs)]
use {{crate_name}}_web::{init_tracing, run};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();

    match run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(
                error.msg = %e,
                error.error_chain = ?e,
                "Shutting down due to error"
            );
            ExitCode::FAILURE
        }
    }
}
