use crate::database::{get_db_conn, Row};
use crate::model::User;
use crate::server::http_response::HttpResponse;
use anyhow::{Error, Result};
use argon2::{self, Config};
use http_auth_basic::Credentials;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use uuid::Uuid;
use warp::http::StatusCode;

#[derive(Deserialize)]
pub struct UserRegister {
    pub name: String,
    pub password: String,
}

pub async fn signup(
    user_register: UserRegister,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    match get_db_conn().await {
        Ok(db_conn) => {
            if let Ok(_) = db_conn
                .query_one(
                    "SELECT * FROM users WHERE users.name = $1 LIMIT 1",
                    &[&user_register.name],
                )
                .await
            {
                return Ok(HttpResponse::new(
                    "Username is taken",
                    StatusCode::BAD_REQUEST,
                ));
            }

            let user_insert_rows: Row = db_conn
                .query_one(
                    "INSERT INTO users(name) VALUES ($1) RETURNING *",
                    &[&user_register.name],
                )
                .await
                .unwrap();

            let user_id: Uuid = user_insert_rows.get(0);
            let hash = make_hash(user_register.password.as_bytes()).unwrap();

            db_conn
                .query(
                    "INSERT INTO secrets(hash, user_id) VALUES ($1, $2)",
                    &[&hash, &user_id],
                )
                .await
                .unwrap();

            let created_user = User {
                id: user_id,
                name: user_insert_rows.get(1),
            };

            Ok(HttpResponse::with_payload(
                created_user,
                StatusCode::CREATED,
            ))
        }
        Err(err) => Ok(HttpResponse::new(
            &format!("An error ocurred!\n{}", err.to_string()),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

pub async fn login(
    auth_header_value: String,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    match Credentials::from_header(auth_header_value) {
        Ok(credentials) => {
            let db_conn = get_db_conn().await.unwrap();

            if let Ok(result_row) = db_conn
                .query_one(
                    r#"
                    SELECT
                        users.id,
                        users.name,
                        secrets.hash
                    FROM
                        users
                        LEFT JOIN secrets ON secrets.user_id = users.id
                    WHERE
                        users.name = $1"#,
                    &[&credentials.user_id],
                )
                .await
            {
                let user_id: Uuid = result_row.get(0);
                let user_name: String = result_row.get(1);
                let user_hash: String = result_row.get(2);

                if verify_hash(&user_hash, credentials.password.as_bytes()) {
                    return Ok(HttpResponse::with_payload(
                        User {
                            id: user_id,
                            name: user_name,
                        },
                        StatusCode::OK,
                    ));
                }

                Ok(HttpResponse::new(
                    "Invalid username/password",
                    StatusCode::FORBIDDEN,
                ))
            } else {
                Ok(HttpResponse::new(
                    "Username doesn't exists",
                    StatusCode::BAD_REQUEST,
                ))
            }
        }
        Err(err) => Ok(HttpResponse::new(&err.to_string(), StatusCode::BAD_REQUEST)),
    }
}

fn make_hash(password: &[u8]) -> Result<String> {
    let conf = Config::default();
    let salt = thread_rng().gen::<[u8; 32]>();

    match argon2::hash_encoded(password, &salt, &conf) {
        Ok(hash) => Ok(hash),
        Err(err) => Err(Error::msg(err.to_string())),
    }
}

fn verify_hash(hash: &str, password: &[u8]) -> bool {
    argon2::verify_encoded(hash, password).unwrap_or(false)
}
