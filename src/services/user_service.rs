use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use log::{error, warn};
use mongodb::error::Error;
use mongodb::Client;

use crate::api::user_api::Pagination;
use crate::constants;
use crate::models::user_list_response::Users;
use crate::{models::error_model::ApiErrorType, models::user_model::User, repository::user_repo};

// add a new user to MongoDB
pub async fn create_user(
    client: &Data<Client>,
    new_user: Json<User>,
) -> Result<HttpResponse, ApiErrorType> {
    let data = User {
        id: None,
        name: new_user.name.to_owned(),
        location: new_user.location.to_owned(),
        title: new_user.title.to_owned(),
    };
    let user_detail = user_repo::create_user(client, data).await;
    match user_detail {
        Ok(Some(user)) => Ok(HttpResponse::Created().json(user)),
        Ok(None) => Err(ApiErrorType::InternalServerError),
        Err(err) => {
            error!("Error: {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

pub async fn get_user_by_id(
    client: &Data<Client>,
    path: Path<String>,
) -> Result<HttpResponse, ApiErrorType> {
    let id = path.into_inner();
    if id.is_empty() {
        warn!("User with id - {} not found for get user by ID", id);
        return Err(ApiErrorType::BadRequest);
    }
    let user_detail = user_repo::get_user(client, &id).await;
    handle_optional_user_response(user_detail)
}

pub async fn update_user(
    client: &Data<Client>,
    path: Path<String>,
    update_user: Json<User>,
) -> Result<HttpResponse, ApiErrorType> {
    let id = path.into_inner();
    if id.is_empty() {
        return Err(ApiErrorType::BadRequest);
    };
    let data = User {
        id: Some(String::from(&id)),
        name: update_user.name.to_owned(),
        location: update_user.location.to_owned(),
        title: update_user.title.to_owned(),
    };

    let update_result = user_repo::update_user(client, &id, data).await;
    match update_result {
        Ok(update) => {
            if update.matched_count == 1 {
                let updated_user_info = user_repo::get_user(client, &id).await;
                handle_optional_user_response(updated_user_info)
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

pub async fn delete_user(
    client: &Data<Client>,
    path: Path<String>,
) -> Result<HttpResponse, ApiErrorType> {
    let id = path.into_inner();
    if id.is_empty() {
        return Err(ApiErrorType::UserNotFound);
    };
    let result = user_repo::delete_user(client, &id).await;
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

pub async fn get_all_users(
    client: &Data<Client>,
    pagination: &Pagination,
) -> Result<HttpResponse, ApiErrorType> {
    let offset = pagination.offset.unwrap_or(constants::DEFAULT_OFFSET_SIZE);
    let limit = pagination.limit.unwrap_or(constants::DEFAULT_LIMIT_SIZE);
    let user_list = user_repo::get_all_users(client, offset, limit).await;
    let user_count = user_repo::get_users_size(client).await.unwrap_or(0);

    let next_offset = i64::try_from(offset).unwrap_or(0) + limit;
    let previous_offset = i64::try_from(offset).unwrap_or(0) - limit;

    match user_list {
        Ok(u) => {
            // TODO: add logic to only show Next and Previous if data is available.
            let mut users = Users {
                href: format!("/api/users?offset={}&limit={}", offset, limit),
                next: None,
                previous: None,
                limit,
                offset,
                total: user_count,
                size: u.len(),
                items: u,
            };
            if next_offset < i64::try_from(user_count).unwrap_or(0) {
                users.next = Some(format!("/api/users?offset={}&limit={}", next_offset, limit));
            }

            if offset > 0 {
                users.previous = Some(format!(
                    "/api/users?offset={}&limit={}",
                    if previous_offset < 0 {
                        0
                    } else {
                        previous_offset
                    },
                    limit
                ));
            }
            Ok(HttpResponse::Ok().json(users))
        }
        Err(err) => {
            error!("Error : {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}

fn handle_optional_user_response(
    user: Result<Option<User>, Error>,
) -> Result<HttpResponse, ApiErrorType> {
    match user {
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(user)),
        Ok(None) => Err(ApiErrorType::UserNotFound),
        Err(err) => {
            error!("Error: {}", err);
            Err(ApiErrorType::InternalServerError)
        }
    }
}