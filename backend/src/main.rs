use std::{sync::Arc, collections::BTreeMap};

use actix_cors::Cors;
use actix_identity::{IdentityMiddleware, Identity};
use actix_session::{storage::CookieSessionStore, SessionMiddleware, config::{BrowserSession, PersistentSession}};
use actix_web::{web::{self, Data}, App, HttpServer, middleware, HttpResponse, dev::{ServiceFactory, ServiceRequest, ServiceResponse}, body::MessageBody, Error, http::header::{self, AUTHORIZATION, REFERER}, cookie::{Key, time::Duration}, HttpRequest, HttpMessage};
use actix_web_httpauth::{middleware::HttpAuthentication, extractors::{basic::BasicAuth, AuthenticationError}, headers::www_authenticate::{WwwAuthenticate, basic::Basic}};
use anyhow::{bail, anyhow};
use common::{StatusResponse, DBPartProps, GetPartProps, DBPart, PartsCategory, CPUProperties};
use surrealdb::{Datastore, Session, sql::Value};
use tokio::sync::Mutex;

const TEN_MINUTES: u64 = 10 * 60;

pub struct DB {
    datastore: Datastore,
    session: Session,
}

async fn status() -> HttpResponse {
    HttpResponse::Ok().json(
        StatusResponse {
            functional: true
        }
    )
}

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
    let sql = "CREATE part CONTENT $props";
    let vars: BTreeMap<String, Value> = [
        ("props".into(), part_props.to_owned().into()),
    ].into();

    let db_locked = db.lock().await;
    let response = db_locked.datastore.execute(sql, &db_locked.session, Some(vars), false).await;
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

async fn logout(id: Identity) -> HttpResponse {
    id.logout();

    HttpResponse::Ok().finish()
}

async fn create_part(id: Option<Identity>, credentials: Option<BasicAuth>, request: HttpRequest, part_props: web::Json<DBPartProps>, db: Data<Mutex<DB>>) -> HttpResponse {
    let mut logged = false;
    if id.is_none() {
        if let Some(credentials) = credentials {
            let user = credentials.user_id();
            let password = credentials.password();
            if let Some(password) = password {
                // TODO: replace with DB lookup
                if user == "Admin" && password == "admin" {
                    Identity::login(&request.extensions(), user.to_string()).unwrap();
                    logged = true;
                }
            }
        }
    } else {
        logged = true;
    }

    if logged {
        let result = create_part_raw(&part_props.0, &db).await;

        return match result {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(_) => HttpResponse::InternalServerError().finish(),
        };
    }

    HttpResponse::Unauthorized()
        .insert_header(WwwAuthenticate::<Basic>(Basic::with_realm("Part creation")))
        .finish()
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
    let secret_key = Key::generate();

    App::new()
        .wrap(middleware::Logger::default())
        .wrap(
            IdentityMiddleware::builder()
                .visit_deadline(Some(std::time::Duration::from_secs(TEN_MINUTES)))
                .build(),
        )
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                .cookie_name("pcpc-auth".to_owned())
                .cookie_secure(false) // We are using HTTP
                .session_lifecycle(BrowserSession::default().state_ttl(Duration::hours(1)))
                .build(),
        )
        .wrap(
            Cors::default()
                //.allowed_origin("http://127.0.0.1:8080")
                .allow_any_origin()
                .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                .allowed_header(header::CONTENT_TYPE)
                .supports_credentials()
                .max_age(3600)
        )
        .app_data(web::JsonConfig::default().limit(4096))
        .app_data(Data::from(db))
        .service(
            web::scope("/api")
                .service(
                    web::resource("")
                        .route(web::get().to(status)),
                )
                .service(
                    web::scope("/part")
                        .service(
                            web::resource("")
                                .route(web::post().to(part)),
                        )
                        .service(
                            web::resource("/create")
                                .route(web::post().to(create_part)),
                        )
                )
                .service(
                    web::resource("/logout")
                        .route(web::post().to(logout)),
                )
        )
}

async fn put_temp_data_to_db(db: Arc<Mutex<DB>>) -> anyhow::Result<()> {
    let db = Data::from(db);

    let temp_parts: Vec<DBPartProps> = vec![
        DBPartProps { 
            name: "Monitor".into(),
            ..Default::default()
        },
        DBPartProps { 
            name: "GPU".into(),
            ..Default::default()
        },
        DBPartProps { 
            name: "CPU".into(),
            ..Default::default()
        },
        DBPartProps { 
            name: "Power Supply".into(),
            ..Default::default()
        },
        DBPartProps { 
            name: "RAM".into(),
            ..Default::default()
        },
        DBPartProps { 
            name: "SSD".into(),
            ..Default::default()
        },
        DBPartProps { 
            name: "HDD".into(),
            ..Default::default()
        },
        DBPartProps { 
            name: "Motherboard".into(),
            ..Default::default()
        },
        DBPartProps { 
            name: "Intel Core i5-13500".into(), 
            image_url: "https://www.intel.com/content/dam/www/central-libraries/xa/en/images/intel-core-i5-badge-1440x1080.png.rendition.intel.web.64.64.png".into(), 
            model: "i5-13500".into(), 
            manufactuer: "Intel".into(), 
            release_date: "22Q4".to_string(), 
            rating: 3.5.into(), 
            category: PartsCategory::CPU(CPUProperties { 
                cores: 14, 
                threads: 20, 
                max_frequency: "4.80 GHz".to_string(),
                base_frequency: "1.80 GHz".to_string(),
                max_tdp: "154 W".to_string(),
                base_tdp: "65 W".to_string(),
                cache: "24 MB".to_string(),
                max_ram_size: "128 GB".to_string(),
                max_memory_channels: 2, 
                ecc_memory_supported: true, 
                max_pcie_lanes: 20, 
                max_supported_pcie_version: "5.0".to_string(), 
                socket: "FCLGA1700".into(), 
                max_temperature: "100 C".to_string(),
            }),
        },
        DBPartProps { 
            name: "Intel Core i5-12500".into(), 
            image_url: "https://www.intel.com/content/dam/www/central-libraries/xa/en/images/intel-core-i5-badge-1440x1080.png.rendition.intel.web.64.64.png".into(), 
            model: "i5-12500".into(), 
            manufactuer: "Intel".into(), 
            release_date: "21Q4".to_string(), 
            rating: 2.5.into(), 
            category: PartsCategory::CPU(CPUProperties { 
                cores: 6, 
                threads: 12, 
                max_frequency: "3.80 GHz".to_string(),
                base_frequency: "1.80 GHz".to_string(),
                max_tdp: "54 W".to_string(),
                base_tdp: "54 W".to_string(),
                cache: "12 MB".to_string(),
                max_ram_size: "128 GB".to_string(),
                max_memory_channels: 2, 
                ecc_memory_supported: true, 
                max_pcie_lanes: 20, 
                max_supported_pcie_version: "4.0".to_string(), 
                socket: "FCLGA1700".into(), 
                max_temperature: "100 C".to_string(),
            }),
        }
    ];

    for temp_part in temp_parts {
        create_part_raw(&temp_part, &db).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let db = create_db_connection().await?;
    put_temp_data_to_db(db.clone()).await?;

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
        let json = DBPartProps {
                    name: part_name.to_owned(),
                    image_url: "".into(),
                    model: "Some model".into(),
                    manufactuer: "AOC".into(),
                    release_date: "23Q1".to_string(),
                    rating: 4.5.into(),
                    category: PartsCategory::Basic,
                };
        let json = serde_json::to_value(&json).unwrap();
        println!("JSON: {}", json);

        let request = 
            test::TestRequest::post()
                .uri("/api/part/create")
                .set_json(json);

        let response = test::call_service(&app, request.to_request()).await;
        let response = response.response();

        println!("Status: {}", response.status());

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
                    .service(
                        web::resource("/api")
                            .route(web::get().to(status)),
                    )
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
