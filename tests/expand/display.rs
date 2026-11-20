#[derive(Display)]
#[display("({x}, {y})", alt = "{x}:{y}")]
struct Point {
    x: u32,
    y: u32,
}

#[derive(Display)]
#[display(inner)]
struct Tuple(u8);

#[derive(Display)]
#[display(Debug)]
struct Debugged<T>(T);

#[derive(Display)]
#[display(Shout::print)]
struct Shout(&'static str);

#[derive(Display)]
#[display("[{0}] {1:?} {width:>2$}")]
struct Generic<T, U> {
    #[display(separator = ", ")]
    items: Vec<T>,
    other: U,
    width: usize,
}

#[derive(Display)]
#[display(doc_comments)]
enum Documented {
    /// Letter A
    A,
    /// Letter {0}
    Letter(char),
    #[display("custom {name}", alt = "{name}")]
    Custom { name: String, other: u8 },
    #[display(lowercase)]
    Lower(u8, u8),
    #[display(inner)]
    Inner(u8),
}

#[derive(Display)]
enum Plain {
    Unit,
    Tuple(u8),
    Named { x: u8 },
}

#[derive(Display)]
enum Never {}

#[derive(Display)]
#[display(uppercase)]
union Bits {
    int: u32,
    float: f32,
}
