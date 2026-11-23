#[derive(From)]
#[from(::std::io::Error)]
struct Unit;

#[derive(From)]
struct Single(u32);

#[derive(From)]
#[from(u8)]
struct Named {
    #[from]
    name: String,
    #[from(u16)]
    count: u32,
}

#[derive(From)]
#[from(u8)]
struct Tuple<T>(u16, #[from] Option<T>);

#[derive(From)]
enum Error {
    #[from(::std::io::Error)]
    #[from(Unit)]
    Io,
    #[from]
    Format(::std::fmt::Error),
    #[from]
    WithFields { details: ::std::str::Utf8Error },
    MultipleFields {
        #[from(Single)]
        io: Named,
        details: String,
    },
    #[from(!)]
    Never,
    #[from(u64)]
    Empty {},
    #[from(i64)]
    Pair(i128, bool),
}

#[derive(From)]
union Either {
    #[from]
    int: u32,
    #[from]
    float: f32,
}
