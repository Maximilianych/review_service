use actix_web::{connect, dev::Server, get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use lazy_static::lazy_static;
use tera::Tera;
use tokio;
use tokio_postgres::{NoTls, Error};


struct WorkingContent<'a> {
    data: &'a str
}
impl<'a> WorkingContent<'a> {
    fn get_data(&self) -> &str {
        &self.data
    }    
}
static WORKING_CONTENT: WorkingContent = WorkingContent{data: "text/html; charset=utf-8"};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*";
        let tera = Tera::new(source).unwrap();
        tera
    };
}


//Service Pages
#[get("/")]
async fn launch() -> impl Responder {
    HttpResponse::Ok().content_type(WORKING_CONTENT.get_data()).body("Главная страница")
}

#[get("/login")]
async fn login() -> impl Responder {
    let page_content = String::from("<h1>Страница авторизации</h1>");
    HttpResponse::Ok().content_type(WORKING_CONTENT.get_data()).body(page_content)
}

#[get("/user")]
async fn user() -> impl Responder {
    HttpResponse::Ok().content_type(WORKING_CONTENT.get_data()).body("Страница пользователя")
}

#[get("/admin")]
async fn admin()-> impl Responder {
    let mut context = tera::Context::new();
    let (client, connection) = tokio_postgres::connect("postgresql://rust:rust@localhost:5432/Service", NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let mut table = String::from("<table class='table table-bordered table-striped'>
                                    <tr>
                                        <th width='7%'>Номер заказа</th>
                                        <th width='35%'>Название книги</th>
                                        <th width='20%'>Автор</th>
                                        <th width='20%'>Рецензер</th>
                                        <th width='10%'>Факультет</th>
                                        <th></th>
                                    </tr>");

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
                        </tr>", row.get(0), row.get(1), row.get(2), row.get(3), row.get(4));
        table.push_str(&tr)
    }
    table.push_str("</table>");
    
    context.insert("table", &table);
    let page_content = TEMPLATES.render("admin.html", &context).unwrap();
    HttpResponse::Ok().body(page_content)
}

struct RunServer {}
impl RunServer {
    fn run() -> Server {
        HttpServer::new(|| {
            App::new()
                .service(launch)
                .service(login)
                .service(user)
                .service(admin)
                .service(fs::Files::new("/assets", "./assets").show_files_listing())
        })
        .bind(("127.0.0.1", 8080)).unwrap()
        .run()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{

    RunServer::run().await?;

    Ok(())
}
