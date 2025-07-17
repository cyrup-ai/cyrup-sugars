//────────────────────────────────────────────────────────────────────────────
// macros – user-facing closure macros for fluent stream/future handling
//────────────────────────────────────────────────────────────────────────────

/// Creates a closure that handles Result values with separate expressions for Ok and Err cases.
/// Available when both 'macros' and any async feature are enabled.
#[macro_export]
macro_rules! on_result {
    (Ok => $ok:expr, Err => $err:expr) => {
        move |__res| match __res {
            Ok(chunk) => Ok($ok),
            Err(err) => Err($err),
        }
    };
}

/// Creates a closure that processes stream chunks with the provided expression.
/// Available when both 'macros' and any async feature are enabled.
#[macro_export]
macro_rules! on_chunk {
    ($expr:expr) => {
        move |__chunk| $expr
    };
}

/// Creates a closure that handles errors with the provided expression.
/// Available when both 'macros' and any async feature are enabled.
#[macro_export]
macro_rules! on_error {
    ($expr:expr) => {
        move |__err| $expr
    };
}

/// Creates an async closure that processes results with the provided pattern and body.
/// Available when both 'macros' and any async feature are enabled.
#[macro_export]
macro_rules! await_result {
    ($param:pat => $body:expr) => {
        move |$param| async move { $body }
    };
}

/// Creates an async closure that processes successful values with the provided pattern and body.
/// Available when both 'macros' and any async feature are enabled.
#[macro_export]
macro_rules! await_ok {
    ($param:pat => $body:expr) => {
        move |$param| async move {
            $body;
        }
    };
}
