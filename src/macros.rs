/// Generates 2 structs, the second serving as a builder pattern for the first
/// # Example
/// ```
/// auto_builder!(
///     /// Love me them doc comments
///     derive(Clone, Copy),
///     MyStruct,
///     MyStructBuilder,
///     field1: char,
///     field2: Option<char>
/// );
/// ```
macro_rules! auto_builder {
    (
        $(#[$($doc_comment:meta),*])?
        $(#[derive($($derive:ident),*)])?
        $vis: vis $name: ident
        $b_vis: vis $builder_name: ident
        $($f_vis:vis $f_name: ident: $f_type: ty),*
    ) => {
        $(#[$($doc_comment),*])?
        $(#[derive($($derive),*)])?
        $vis struct $name {
            $($f_vis $f_name: $f_type),*
        }

        $(#[$($doc_comment),*])?
        $(#[derive($($derive),*)])?
        $b_vis struct $builder_name {
            $($f_name: $f_type),*
        }
    };
}

// macros
macro_rules! ignorant_queue {
    ($writer:expr $(, $command:expr)* $(,)?) => {
        let _ = $crate::queue!($writer() $(, $command)*);
    };
}

macro_rules! ignorant_execute {
    ($writer:expr $(, $command:expr)* $(,)?) => {
        let _ = $crate::execute!($writer() $(, $command)*);
    };
}

macro_rules! ignorant_flush {
    ($writer:expr) => {
        let _ = $writer().flush();
    };
}