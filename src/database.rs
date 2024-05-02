use std::sync::Once;
use postgres::{Client, NoTls};

pub static mut DATABASE: Database = Database::new(|| Client::connect("postgresql://rust:rust@localhost:5432/Service", NoTls).unwrap());
pub static ONCE: Once = Once::new();

pub struct Database {
    client: Option<Client>,
    init: fn() -> Client,
}

impl Database {
    pub const fn new(init: fn() -> Client) -> Self {
        Database { client: None, init }
    }

    pub fn instance(&mut self) -> &mut Client {
        ONCE.call_once(|| {
            let init = self.init;
            let client = init();

            unsafe{
                DATABASE.client = Some(client);
            }
            println!("Синглетонов вход в БД готов");
        });
        println!("Синглетонов вход используется");
        let instance = &mut self.client;
        instance.as_mut().unwrap()
    }
}


