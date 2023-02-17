use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{get, web::{self, Data}, App, HttpServer, middleware, HttpResponse, dev::{ServiceFactory, ServiceRequest, ServiceResponse}, body::MessageBody, Error, post, http::header};
use anyhow::{bail, anyhow};
use common::{StatusResponse, DBPartProps, GetPartProps, DBPart};
use surrealdb::{Datastore, Session};
use tokio::sync::Mutex;

pub struct DB {
    datastore: Datastore,
    session: Session,
}

#[get("/api")]
async fn status() -> HttpResponse {
    HttpResponse::Ok().json(
        StatusResponse {
            functional: true
        }
    )
}

#[post("/api/part")]
async fn part(props: web::Json<GetPartProps>, db: Data<Mutex<DB>>) -> HttpResponse {
    let props = props.into_inner();

    let db_locked = db.lock().await;
    let props_id = props.id.clone();
    let sql;
    if let Some(part_id) = props_id {
        sql = format!("SELECT * FROM part:{}", part_id);
    } else {
        sql = format!("SELECT * FROM part LIMIT {}", props.limit);
    }

    let response = db_locked.datastore.execute(sql.as_str(), &db_locked.session, None, false).await;

    drop(db_locked);

    if let Err(_) = response {
        HttpResponse::InternalServerError().finish();
    }

    let response = response.unwrap();

    for response in response {
        if let Ok(result) = response.result {
            let json = serde_json::to_value(result.clone()).unwrap();
            let mut parts: Vec<DBPart> = serde_json::from_value(json).unwrap();

            if parts.len() == 0 {
                return HttpResponse::NotFound().finish();
            }

            for part in parts.iter_mut() {
                part.id = part.id.replace("part:", "");
            }

            if props.id.is_some() {
                if parts.len() > 1 {
                    return HttpResponse::InternalServerError().finish();
                }

                let part = parts.first().unwrap().clone();

                return HttpResponse::Ok().json(part);
            }

            return HttpResponse::Ok().json(parts);
        }

        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::InternalServerError().finish()
}

async fn create_db_connection() -> anyhow::Result<Arc<Mutex<DB>>> {
    let datastore = Datastore::new("memory").await?;
    let session = Session::for_db("my_ns", "my_db");

    Ok(Arc::new(Mutex::new(DB {
        datastore,
        session,
    })))
}

async fn create_part_raw(part_props: &DBPartProps, db: &Data<Mutex<DB>>) -> anyhow::Result<()> {
    let sql = format!("CREATE part SET name = '{}'", part_props.name);
    let db_locked = db.lock().await;
    let response = db_locked.datastore.execute(sql.as_str(), &db_locked.session, None, false).await;

    drop(db_locked);

    if let Err(error) = response { 
        bail!(error);
    }

    let response = response.unwrap();

    if let Some(first) = response.first() {
        match &first.result {
            Ok(_) => { return Ok(()) },
            Err(error) => { return Err(anyhow!("DB error: {}", error.to_string())); },
        }
    }

    Ok(())
}

#[post("/api/part/create")]
async fn create_part(part_props: web::Json<DBPartProps>, db: Data<Mutex<DB>>) -> HttpResponse {
    let result = create_part_raw(&part_props.0, &db).await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

fn create_app(
    db: Arc<Mutex<DB>>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    App::new()
        .wrap(middleware::Logger::default())
        .wrap(
            Cors::default()
                .allowed_origin("http://127.0.0.1:8080")
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                .allowed_header(header::CONTENT_TYPE)
                .supports_credentials()
                .max_age(3600)
        )
        .app_data(web::JsonConfig::default().limit(4096))
        .app_data(Data::from(db))
        .service(status)
        .service(create_part)
        .service(part)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = create_db_connection().await?;

    HttpServer::new(move || {
        create_app(db.clone())
    })
    .bind(("127.0.0.1", 8088))?
    .run()
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use actix_web::{test, dev::Service, http::{self, StatusCode}};
    use common::DBPart;
    use super::*;

    async fn create_local_db() -> anyhow::Result<Arc<Mutex<DB>>> {
        let datastore = Datastore::new("memory").await?;
        let session = Session::for_db("my_ns", "my_db");

        Ok(Arc::new(Mutex::new(DB { 
            datastore, 
            session,
        })))
    }

    #[actix_web::test]
    async fn test_db() {
        let db = create_local_db().await;

        assert!(!db.is_err());

        let db = db.unwrap();

        let app =
            test::init_service(create_app(db.clone()))
            .await;

        let part_name = "Monitor".to_string();

        let request = 
            test::TestRequest::post()
                .uri("/api/part/create")
                .set_json(
                    DBPartProps {
                        name: part_name.to_owned(),
                    }
                );

        let response = test::call_service(&app, request.to_request()).await;
        let response = response.response();

        assert!(match response.status() {
            StatusCode::OK => true,
            _ => false,
        });

        let request = 
            test::TestRequest::post()
                .uri("/api/part")
                .set_json(
                    GetPartProps {
                        id: None,
                        limit: 1,
                    }
                );

        let response: Vec<DBPart> = test::call_and_read_body_json(&app, request.to_request()).await;
        assert!(response.len() == 1);

        let first_part = response.first().unwrap();
        assert!(part_name == first_part.name);

        let request = 
            test::TestRequest::post()
                .uri("/api/part")
                .set_json(
                    GetPartProps {
                        id: Some(first_part.id.clone()),
                        limit: 1,
                    }
                );

        let response: DBPart = test::call_and_read_body_json(&app, request.to_request()).await;
        assert!(*first_part == response);
    }

    #[actix_web::test]
    async fn test_status() {
        let app = 
            test::init_service(
                App::new()
                    .service(status)
            )
            .await;

        let request = 
            test::TestRequest::get()
            .uri("/api")
            .to_request();

        let responce = app.call(request).await.unwrap();

        assert_eq!(responce.status(), http::StatusCode::OK);
    }
}
