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

/// Creates a closure that processes Result<T, E> where E: Into<T> with elegant pattern matching.
/// Both Ok and Err branches return type T directly for zero-allocation stream processing.
/// Available when both 'macros' and any async feature are enabled.
#[macro_export]
macro_rules! on_chunk {
    (|$chunk:ident| {
        Ok => $ok_expr:expr,
        Err($err:ident) => $err_expr:expr
    }) => {
        |$chunk| match $chunk {
            Ok($chunk) => $ok_expr,
            Err($err) => $err_expr
        }
    };
    (|$chunk:ident| {
        Ok => $ok_block:block,
        Err($err:ident) => $err_expr:expr
    }) => {
        |$chunk| match $chunk {
            Ok($chunk) => $ok_block,
            Err($err) => $err_expr
        }
    };
    (|$chunk:ident| {
        Ok => $ok_expr:expr,
        Err($err:ident) => $err_block:block
    }) => {
        |$chunk| match $chunk {
            Ok($chunk) => $ok_expr,
            Err($err) => $err_block
        }
    };
    (|$chunk:ident| {
        Ok => $ok_block:block,
        Err($err:ident) => $err_block:block
    }) => {
        |$chunk| match $chunk {
            Ok($chunk) => $ok_block,
            Err($err) => $err_block
        }
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