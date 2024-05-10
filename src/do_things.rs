use tera::Context;
use tokio_postgres::Client;

use crate::NewOrderForm;

pub struct DoThings {}
impl DoThings {
    pub async fn do_admin_table(client: &Client, context: &mut Context) {
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
                                                ON request.faculty_id = faculty.faculty_id", &[])
                                                .await
                                                .unwrap() {
            let tr = format!("<tr>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>
                                            <a hx-delete='/delete_order/{}' hx-target='closest tr' hx-swap='outerHTML' class='btn-outline-dark btn'><i class='bi-trash3-fill'></i></a>
                                        </td>
                                    </tr>", row.get::<usize, &str>(0), row.get::<usize, &str>(1), row.get::<usize, &str>(2), row.get::<usize, &str>(3), row.get::<usize, &str>(4), row.get::<usize, &str>(0));
            table.push_str(&tr)
        }
        table.push_str("</table>");
        context.insert("table", &table);
    }

    pub async fn do_user_table(id: u32, client: &Client) -> String {
        let mut table = String::from(
            "<table class='table table-bordered table-striped col-6'>
            <tr>
                <th width='50%'>Название книги</th>
                <th width='30%'>Рецензер</th>
                <th width='10%'>Факультет</th>
                <th></th>
            </tr>",
        );

        for row in client.query("SELECT CAST(request_id as varchar(10)), book_name, reviewer_name, faculty_name FROM request
                                                INNER JOIN reviewer
                                                ON request.reviewer_id = reviewer.reviewer_id
                                                INNER JOIN faculty
                                                ON request.faculty_id = faculty.faculty_id
                                                WHERE request.author_id = $1", &[&(id as i32)])
                                                .await
                                                .unwrap() {
            let tr = format!("<tr>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>{}</td>
                                        <td>
                                            <a hx-delete='/delete_order/{}' hx-target='closest tr' hx-swap='outerHTML' class='btn-outline-dark btn'><i class='bi-trash3-fill'></i></a>
                                        </td>
                                    </tr>", row.get::<usize, &str>(1), row.get::<usize, &str>(2), row.get::<usize, &str>(3), row.get::<usize, &str>(0));
            table.push_str(&tr)
        }
        table.push_str("</table>");
        return table;
    }

    pub async fn reviewers_from_faculty(facult: i32, client: &Client, context: &mut Context) {
        let mut option: String = String::default();
        for row in client
            .query(
                "SELECT reviewer_id, reviewer_name FROM reviewer WHERE reviewer_faculty_id = $1",
                &[&facult],
            )
            .await
            .unwrap()
        {
            option.push_str(&format!(
                "<option value='{}'>{}</option>\n",
                row.get::<usize, i32>(0),
                row.get::<usize, &str>(1)
            ));
            println!("Option: {}", option);
        }
        context.insert("form_reviewer", &option);
    }

    pub async fn create_order(order_form: &NewOrderForm, client: &Client) -> bool {
        match client
            .execute(
                "INSERT INTO request(book_name, author_id, reviewer_id, faculty_id)
                                VALUES ($1, $2, $3, $4);",
                &[
                    &order_form.book_name,
                    &order_form.user_id,
                    &order_form.reviewer,
                    &order_form.facult,
                ],
            )
            .await
        {
            Ok(_) => {
                return true;
            }
            Err(e) => {
                println!("Удаление ничего не удалило, где-то ошибочка: {}", e);
                return false;
            }
        }
    }

    pub async fn delete_order(id: u32, client: &Client) -> bool {
        match client
            .execute("DELETE FROM request WHERE request_id = $1", &[&(id as i32)])
            .await
        {
            Ok(_) => {
                println!("Удаление удалило заказ с номером {}", id);
                return true;
            }
            Err(e) => {
                println!("Удаление ничего не удалило, где-то ошибочка: {}", e);
                return false;
            }
        }
    }
}
