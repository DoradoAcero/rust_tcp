// continue if it fails
#[macro_export]
macro_rules! unwrap_or_continue {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(err) => {
                println!("continuing because null {}", err);
                continue;
            }
        }
    };
}