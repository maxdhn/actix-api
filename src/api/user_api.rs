use crate::{
    models::{error_model::ApiErrorType, user_model::User},
    repository::mongodb_repo::MongoRepo,
};
use actix_web::{
    delete, get, post, put, web,
    web::{Data, Json, Path},
    HttpResponse,
};
use log::error;
use log::info;
use log::warn;
use validator::Validate;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(create_user);
    cfg.service(get_user);
    cfg.service(update_user);
    cfg.service(delete_user);
    cfg.service(get_all_users);
}

#[post("/users")]
pub async fn create_user(
    db: Data<MongoRepo>,
    new_user: Json<User>,
) -> Result<HttpResponse, ApiErrorType> {
    let is_valid = new_user.validate();
    match is_valid {
        Ok(_) => {
            info!("creating a new user with name - {}", new_user.name);
            let data = User {
                id: None,
                name: new_user.name.to_owned(),
                location: new_user.location.to_owned(),
                title: new_user.title.to_owned(),
            };
            let user_detail = db.create_user(data).await;
            match user_detail {
                Ok(user) => Ok(HttpResponse::Created().json(user)),
                Err(err) => {
                    error!("Error: {}", err);
                    Err(ApiErrorType::InternalServerError)
                }
            }
        }
        Err(err) => {
            warn!("Error: {}", err);
            // Validation error.
            Err(ApiErrorType::BadRequest)
        }
    }
}

#[get("/users/{id}")]
pub async fn get_user(
    db: Data<MongoRepo>,
    path: Path<String>,
) -> Result<HttpResponse, ApiErrorType> {
    let id = path.into_inner();
    if id.is_empty() {
        warn!("User with id -{} not found for get user by ID", id);
        return Err(ApiErrorType::BadRequest);
    }
    let user_detail = db.get_user(&id).await;
    match user_detail {
        Ok(Some(user)) => Ok(HttpResponse::Created().json(user)),
        Ok(None) => Err(ApiErrorType::UserNotFound),
        Err(err) => {
            error!("Error: {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

#[put("/users/{id}")]
pub async fn update_user(
    db: Data<MongoRepo>,
    path: Path<String>,
    new_user: Json<User>,
) -> Result<HttpResponse, ApiErrorType> {
    let id = path.into_inner();
    if id.is_empty() {
        return Err(ApiErrorType::BadRequest);
    };
    let data = User {
        id: Some(String::from(&id)),
        name: new_user.name.to_owned(),
        location: new_user.location.to_owned(),
        title: new_user.title.to_owned(),
    };
    let update_result = db.update_user(&id, data).await;
    match update_result {
        Ok(update) => {
            if update.matched_count == 1 {
                let updated_user_info = db.get_user(&id).await;
                match updated_user_info {
                    Ok(Some(user)) => Ok(HttpResponse::Ok().json(user)),
                    Ok(None) => Err(ApiErrorType::UserNotFound),
                    Err(err) => {
                        error!("Error: {}", err);
                        Err(ApiErrorType::InternalServerError)
                    }
                }
            } else {
                warn!("User with id -{} not found update user by ID", id);
                Err(ApiErrorType::UserNotFound)
            }
        }
        Err(err) => {
            error!("Error: {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

#[delete("/users/{id}")]
pub async fn delete_user(
    db: Data<MongoRepo>,
    path: Path<String>,
) -> Result<HttpResponse, ApiErrorType> {
    let id = path.into_inner();
    if id.is_empty() {
        return Err(ApiErrorType::UserNotFound);
    };
    let result = db.delete_user(&id).await;
    match result {
        Ok(res) => {
            if res.deleted_count == 1 {
                Ok(HttpResponse::NoContent().finish())
            } else {
                warn!("User with id -{} not found for delete user by ID", id);
                Err(ApiErrorType::UserNotFound)
            }
        }
        Err(err) => {
            error!("Error : {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

#[get("/users")]
pub async fn get_all_users(db: Data<MongoRepo>) -> Result<HttpResponse, ApiErrorType> {
    let users = db.get_all_users().await;
    match users {
        Ok(users) => Ok(HttpResponse::Ok().json(users)),
        Err(err) => {
            error!("Error : {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}
