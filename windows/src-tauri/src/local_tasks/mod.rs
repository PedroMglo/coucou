//! Desktop Ollama tools. No command can enter the executor without a native,
//! one-use approval and a durable starting receipt for this exact chat turn.
pub(crate) mod approval;
pub(crate) mod executor;
pub(crate) mod protocol;
pub(crate) mod run;
pub(crate) mod types;

pub(crate) use run::respond;
