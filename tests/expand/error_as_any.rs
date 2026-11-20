#[derive(Error)]
struct Failure<T: Debug>(T);

#[derive(AsAny)]
struct Any;

#[derive(AsAny)]
#[amplify_crate(renamed)]
struct Renamed<T>(T);
