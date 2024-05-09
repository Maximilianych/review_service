use tokio_postgres::NoTls;

pub struct Chain {}
impl Chain {
    pub async fn check_user(username: &String, password: &String) -> bool {
        let user = ChainEmail::check_mail(username).await;
        match user {
            Some(User::Email) => ChainEmail::check_password(username, password).await,
            Some(User::Login) => ChainLogin::check_password(username, password).await,
            Some(User::Phone) => ChainPhone::check_password(username, password).await,
            None => {println!("Увы, но что-то не так"); false}
        }
    }
}

pub enum User {
    Email,
    Login,
    Phone,
}

pub struct ChainEmail {}
impl ChainEmail {
    pub async fn check_mail(username: &str) -> Option<User> {
        println!("В check_mail");
        let (client, connection) =
            tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
                .await
                .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let result = client
            .query(
                "SELECT * FROM person_data
                WHERE person_mail = $1",
                &[&username],
            )
            .await.unwrap();
        let mut result = result.iter(); //-----------ITERATOR------------
        match result.next() {
            Some(_) => {println!("Это почта, можно больше не продолжать, {}", &username); Some(User::Email)},
            None => {println!("Это не почта, продолжаем"); ChainLogin::check_login(username).await},
        }
    }

    pub async fn check_password(username: &str, password: &str) -> bool {
        let (client, connection) =
            tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
                .await
                .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let result = client
            .query(
                "SELECT * FROM person_data
                WHERE person_mail = $1
                AND person_password = $2",
                &[&username, &password],
            )
            .await;
        match result {
            Ok(_) => {println!("Пароль что-надо, кайфуем"); true},
            Err(_) => false
        }
    }
}

pub struct ChainLogin {}
impl ChainLogin {
    pub async fn check_login(username: &str) -> Option<User> {
        let (client, connection) =
            tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
                .await
                .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let result = client
            .query(
                "SELECT * FROM person_data
                WHERE person_login = $1",
                &[&username],
            )
            .await.unwrap();
        let mut result = result.iter();
        match result.next() {
            Some(_) => {println!("Это логин, можно больше не продолжать"); Some(User::Login)},
            None => {println!("Это не логин, продолжаем"); ChainPhone::check_phone(username).await},
        }
    }

    pub async fn check_password(username: &str, password: &str) -> bool {
        let (client, connection) =
            tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
                .await
                .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let result = client
            .query(
                "SELECT * FROM person_data
                WHERE person_login = $1
                AND person_password = $2",
                &[&username, &password],
            )
            .await;
        match result {
            Ok(_) => {println!("Пароль что-надо, кайфуем"); true},
            Err(_) => false
        }
    }
}


pub struct ChainPhone {}
impl ChainPhone {
    pub async fn check_phone(username: &str) -> Option<User> {
        let (client, connection) =
            tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
                .await
                .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let result = client
            .query(
                "SELECT * FROM person_data
                WHERE person_phone = $1",
                &[&username],
            )
            .await.unwrap();
        let mut result = result.iter();
        match result.next() {
            Some(_) => {println!("Это телефон, можно больше не продолжать"); Some(User::Phone)},
            None => {println!("Это не телефон, продолжать некуда"); None}
        }
    }

    pub async fn check_password(username: &str, password: &str) -> bool {
        let (client, connection) =
            tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
                .await
                .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let result = client
            .query(
                "SELECT * FROM person_data
                WHERE person_phone = $1
                AND person_password = $2",
                &[&username, &password],
            )
            .await;
        match result {
            Ok(_) => {println!("Пароль что-надо, кайфуем"); true},
            Err(_) => false
        }
    }
}