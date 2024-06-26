use actix_files as fs;
use actix_web::{dev::Server, get, post, delete, web, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use serde::Deserialize;
use tera::Tera;
use tokio;
use tokio_postgres::NoTls;

mod chain;
mod do_things;
use chain::Chain;
use do_things::DoThings;

use std::env;

//-------------Singleton--------------
lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*";
        let tera = Tera::new(source).unwrap();
        tera
    };
}

//Service Pages
#[get("/")] //------------Decorator----------------
async fn start() -> impl Responder {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("index.html", &context).unwrap();
    HttpResponse::Ok()
        .insert_header(("HX-redirect", "/"))
        .body(page_content)
}

#[get("/login")]
async fn login() -> impl Responder {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("login.html", &context).unwrap();
    HttpResponse::Ok()
        .append_header(("HX-redirect", "/login"))
        .body(page_content)
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[post("/login")]
async fn login_post(data: web::Form<LoginForm>) -> impl Responder {
    let (client, connection) =
        tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
            .await
            .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    if Chain::check_user(&data.username, &data.password).await { //----------Chain of Responsibility--------------
        let id = client
            .query(
                "SELECT person_id FROM person_data
                WHERE person_mail = $1
                OR person_login = $1
                OR person_phone = $1",
                &[&data.username],
            )
            .await
            .unwrap();
        let id = id.iter().next().unwrap();
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

#[get("/user/{id}")]
async fn user(id: web::Path<(u32,)>) -> impl Responder {
    let id = id.into_inner().0;
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
    context.insert("user_id", &id);
    let table = DoThings::do_user_table(id, &client).await;
    context.insert("table", &table);
    let page_content = TEMPLATES.render("user.html", &context).unwrap();
    HttpResponse::Ok().body(page_content)
}

#[derive(Debug)]
#[derive(Deserialize)]
struct NewOrderForm {
    user_id: i32,
    book_name: String,
    facult: i32,
    reviewer: i32,
}

#[post("/changing_facult")]
async fn changing_facult(order_form: web::Form<NewOrderForm>) -> impl Responder {
    let (client, connection) =
    tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("Факультет моменян на {}", order_form.facult);
    let mut context = tera::Context::new();

    DoThings::reviewers_from_faculty(order_form.facult, &client, &mut context).await;

    let reviewer_content = TEMPLATES.render("form_reviewer.html", &context).unwrap();

    HttpResponse::Ok().body(reviewer_content)
}

#[post("/new_order_user")]
async fn new_order_user(order_form: web::Form<NewOrderForm>) -> impl Responder {
    let (client, connection) =
    tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("Новый заказ {:?}", order_form);
    let order_form = order_form.into_inner();

    if DoThings::create_order(&order_form, &client).await {
        return HttpResponse::Ok().append_header(("HX-redirect", format!("/user/{}", order_form.user_id))).finish();
    }
    else {
        return HttpResponse::BadRequest().finish();
    }
}

#[post("/new_order_admin")]
async fn new_order_admin(order_form: web::Form<NewOrderForm>) -> impl Responder {
    let (client, connection) =
    tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("Новый заказ {:?}", order_form);
    let order_form = order_form.into_inner();

    if DoThings::create_order(&order_form, &client).await {
        return HttpResponse::Ok().append_header(("HX-redirect", "/admin")).finish();
    }
    else {
        return HttpResponse::BadRequest().finish();
    }
}

#[delete("/delete_order/{id}")]
async fn delete_order(id: web::Path<(u32,)>) -> impl Responder {
    let (client, connection) =
        tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
            .await
            .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let id = id.into_inner().0;

    if DoThings::delete_order(id, &client).await {
        HttpResponse::Ok()
    }
    else {
        HttpResponse::BadRequest()
    }
}

#[post("/switch_have_review/{id}")]
async fn switch_have_review(id: web::Path<(u32,)>) -> impl Responder {
    println!("Меняю статус заказа");
    let (client, connection) =
        tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls)
            .await
            .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let id = id.into_inner().0;

    let statement = format!("UPDATE request
                                    SET have_review = (SELECT have_review # 1 FROM request WHERE request_id = {id}) 
                                    WHERE request_id = {id}");
    client.execute(&statement, &[]).await.unwrap();
    HttpResponse::Ok().append_header(("HX-redirect", "/admin")).finish()
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

    DoThings::do_admin_table(&client, &mut context).await;
    let page_content = TEMPLATES.render("admin.html", &context).unwrap();
    HttpResponse::Ok()
        .append_header(("HX-redirect", "/admin"))
        .body(page_content)
}

struct RunServer {}
impl RunServer {
    fn run() -> Server {
        HttpServer::new(|| {
            App::new()
                .service(start)
                .service(login)
                .service(login_post)
                .service(user)
                .service(changing_facult)
                .service(admin)
                .service(new_order_user)
                .service(new_order_admin)
                .service(delete_order)
                .service(switch_have_review)
                .service(fs::Files::new("/assets", "./assets").show_files_listing())
        })
        .bind(("127.0.0.1", 8080))
        .unwrap()
        .run()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");

    RunServer::run().await?;

    Ok(())
}
