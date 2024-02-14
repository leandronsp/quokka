use pool::{Pool, Dirty, Checkout};
use postgres::{Client, NoTls};

pub fn connection() -> Checkout<Dirty<Client>> {
    let mut pool = Pool::with_capacity(10, 0, || Dirty(pg_connection()));

    return pool.checkout().unwrap()
}

pub fn pg_connection() -> Client {
    return Client
        ::connect("host=localhost user=postgres password=postgres dbname=postgres", NoTls)
        .unwrap()
}
