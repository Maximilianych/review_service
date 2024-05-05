use actix_files as fs;
use actix_web::{
    dev::Server,
    get, post,
    web,
    App, HttpResponse, HttpServer, Responder,
};
use lazy_static::lazy_static;
use serde::Deserialize;
use tera::{Context, Tera};
use tokio;
use tokio_postgres::{Client, NoTls};

mod chain;
use chain::Chain;

use std::{env, path::Path};

struct WorkingContent<'a> {
    data: &'a str,
}
impl<'a> WorkingContent<'a> {
    fn get_data(&self) -> &str {
        &self.data
    }
}
static WORKING_CONTENT: WorkingContent = WorkingContent {
    data: "text/html; charset=utf-8",
};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*";
        let tera = Tera::new(source).unwrap();
        tera
    };
}

struct TableTransformer {}
impl TableTransformer {
    async fn do_admin_table(client: &Client, context: &mut Context) {
        let mut table = String::from(
            "<table class='table table-bordered table-striped'>
            <tr>
                <th width='7%'>Номер заказа</th>
                <th width='35%'>Название книги</th>
                <th width='20%'>Автор</th>
                <th width='20%'>Рецензер</th>
                <th width='10%'>Факультет</th>
                <th></th>
            </tr>",
        );
        for row in client.query("SELECT CAST(request_id as varchar(10)), book_name, person_name, reviewer_name, faculty_name FROM request
                                    INNER JOIN person
                                    ON request.author_id = person.person_id
                                    INNER JOIN reviewer
                                    ON request.reviewer_id = reviewer.reviewer_id
                                    INNER JOIN faculty
                                    ON request.faculty_id = faculty.faculty_id", &[]).await.unwrap() {
            let tr = format!("<tr>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>
                                            <a class='btn-outline-dark btn'><i class='bi-trash3-fill'></i></a>
                                        </td>
                                    </tr>", row.get::<usize, &str>(0), row.get::<usize, &str>(1), row.get::<usize, &str>(2), row.get::<usize, &str>(3), row.get::<usize, &str>(4));
            table.push_str(&tr)
        }
        table.push_str("</table>");
        context.insert("table", &table);
    }

    async fn do_user_table(id: u32, client: &Client, context: &mut Context) {
        let mut table = String::from(
            "<table class='table table-bordered table-striped col-6'>
            <tr>
                <th width='50%'>Название книги</th>
                <th width='30%'>Рецензер</th>
                <th width='10%'>Факультет</th>
                <th></th>
            </tr>",
        );
        let id = id as i32;
        for row in client.query("SELECT CAST(request_id as varchar(10)), book_name, reviewer_name, faculty_name FROM request
                                                INNER JOIN reviewer
                                                ON request.reviewer_id = reviewer.reviewer_id
                                                INNER JOIN faculty
                                                ON request.faculty_id = faculty.faculty_id
                                                WHERE request.author_id = $1", &[&id]).await.unwrap() {
            let tr = format!("<tr>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>
                                            <a class='btn-outline-dark btn'><i class='bi-trash3-fill'></i></a>
                                        </td>
                                    </tr>", row.get::<usize, &str>(1), row.get::<usize, &str>(2), row.get::<usize, &str>(3));
            table.push_str(&tr)
        }
        table.push_str("</table>");
        context.insert("table", &table);
    }
}

//Service Pages
#[get("/")]
async fn launch() -> impl Responder {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("index.html", &context).unwrap();
    HttpResponse::Ok().body(page_content)
}

#[get("/login")]
async fn login() -> impl Responder {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("login.html", &context).unwrap();
    HttpResponse::Ok().append_header(("HX-redirect", "/login")).body(page_content)
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

//-------------CHAIN------------
#[post("/login")]
async fn login_post(data: web::Form<LoginForm>) -> impl Responder {
    println!("username: {}, password: {}", data.username, data.password);
    let (client, connection) =
        tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
            .await
            .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    if Chain::check_user(&data.username, &data.password).await {
        let id = client.query(
                "SELECT person_id FROM person_data
                                         WHERE person_mail = $1
                                         OR person_login = $1
                                         OR person_phone = $1",
                &[&data.username],
            )
            .await
            .unwrap();
        let id = id.iter().next().unwrap(); //------------ITERATOR---------------
        println!("ID: {}", id.get::<usize, i32>(0));
        HttpResponse::Ok()
            .append_header(("HX-Redirect", format!("/user/{}", id.get::<usize, i32>(0))))
            .finish()
    } else {
        let context = tera::Context::new();
        let content = TEMPLATES.render("login_fail.html", &context).unwrap();
        HttpResponse::Ok().body(content)
    }
}
//---------------CHAIN--------------

#[get("/user/{id}")]
async fn user(id: web::Path<(u32,)>) -> impl Responder {
    let (client, connection) =
        tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
            .await
            .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let mut context = tera::Context::new();

    TableTransformer::do_user_table(id.into_inner().0, &client, &mut context).await;

    let page_content = TEMPLATES.render("user.html", &context).unwrap();
    HttpResponse::Ok().body(page_content)
}

#[get("/admin")]
async fn admin() -> impl Responder {
    let (client, connection) =
        tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
            .await
            .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let mut context = tera::Context::new();

    TableTransformer::do_admin_table(&client, &mut context).await;

    let page_content = TEMPLATES.render("admin.html", &context).unwrap();
    HttpResponse::Ok().append_header(("HX-redirect", "/admin")).body(page_content)
}

struct RunServer {}
impl RunServer {
    fn run() -> Server {
        HttpServer::new(|| {
            App::new()
                .service(launch)
                .service(login)
                .service(login_post)
                .service(user)
                .service(admin)
                .service(fs::Files::new("/assets", "./assets").show_files_listing())
        })
        .bind(("127.0.0.1", 8080))
        .unwrap()
        .run()
    }
}

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    env::set_var("RUST_BACKTRACE", "0");

    RunServer::run().await?;

    Ok(())
}
