use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use elerp_common::{model::{ListSlice, Pagination}, user_system::model::{user_configure::UserConfigure, user_info::{GetUsersQuery, UserInfo, UserType}}};
use serde_qs::axum::QsQuery as Query;
use sqlx::SqliteConnection;
use utoipa::OpenApi;

use crate::custom_error::{AppError, CustomErrorCode};

use super::{
    model::authenticated_user::{AuthenticatedUser, GetTokenQuery},
    AppState,
};

type Result<T> = core::result::Result<T, AppError>;

#[derive(OpenApi)]
#[openapi(
    paths(
        add_user,
        remove_user,
        get_user,
        get_users,
        update_user,
        get_user_token,
    ),
    tags(
        (name = "UserSystem", description = "User System API")
    ),
    components(
        schemas(
            UserInfo,
            GetUsersQuery,
            GetTokenQuery,
            AuthenticatedUser,
        )
    )
)]
pub struct ApiDoc;

pub fn get_services() -> Router<AppState> {
    Router::new()
        .route("/users", post(add_user).get(get_users))
        .route(
            "/users/:id",
            delete(remove_user).get(get_user).put(update_user),
        )
        .route("/users_token", get(get_user_token))
        .route("/users_token/:id", delete(remove_user_token))
        .route(
            "/users_configure/:id",
            get(get_user_configure).put(update_user_configure),
        )
        .route("/me", get(get_me_by_token))
}

async fn check_user(
    s: AppState,
    v: &UserInfo,
    prev: Option<i64>,
    tx: &mut SqliteConnection,
) -> Result<()> {
    if s.us.is_exists_name(&v.username, prev, &mut *tx).await? {
        return AppError::custom(
            CustomErrorCode::SameObject,
            "Already contains the user's name.",
        )
        .into_err();
    }
    Ok(())
}

/// add user
#[utoipa::path(
    post,
    path = "/users",
    responses(
        (status = 200, description = "add user successfully", body = UserInfo)
    ),
)]
async fn add_user(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(body): Json<UserInfo>,
) -> Result<Json<UserInfo>> {
    authenticated.fail_if_not_admin()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.us.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(
            CustomErrorCode::UserLimitExceeded,
            "User count limit exceeded!",
        )
        .into_err();
    }
    check_user(s.clone(), &body, None, tx.as_mut()).await?;
    if body.user_type == UserType::Admin {
        return AppError::custom(
            CustomErrorCode::NoPermission,
            "You don't have permission add the admin.",
        )
        .into_err();
    }
    let v = s.us.add_user(body, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

/// remove user
#[utoipa::path(
    delete,
    path = "/users/{id}",
    responses(
        (status = 200, description = "remove user successfully")
    ),
    params(
        ("id" = i64, Path, description = "user id")
    )
)]
async fn remove_user(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    authenticated.fail_if_not_admin()?;
    if id == authenticated.user.id {
        return AppError::custom(CustomErrorCode::NoPermission, "You can't remove yourself!")
            .into_err();
    }
    let mut tx = s.ps.begin_tx(true).await?;
    let r = s.us.remove_user(id, tx.as_mut()).await?;
    tx.commit().await?;
    if r {
        Ok(StatusCode::OK)
    } else {
        AppError::custom(CustomErrorCode::UserNotFound, "User is not exists.").into_err()
    }
}

/// get user by id
#[utoipa::path(
    get,
    path = "/users/{id}",
    responses(
        (status = 200, description = "get user successfully", body = UserInfo)
    ),
    params(
        ("id" = i64, Path, description = "user id")
    )
)]
async fn get_user(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<Json<UserInfo>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let r =
        s.us.get_user(id, authenticated.user.as_action_type(false), tx.as_mut())
            .await?;
    tx.commit().await?;
    if let Some(v) = r {
        Ok(Json(v))
    } else {
        AppError::custom(CustomErrorCode::UserNotFound, "User is not exists.").into_err()
    }
}

/// get users
#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "get user successfully", body = ListSlice<UserInfo>)
    ),
    params(
        Pagination,
        GetUsersQuery,
    )
)]
async fn get_users(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(query): Query<GetUsersQuery>,
    authenticated: AuthenticatedUser,
) -> Result<Json<ListSlice<UserInfo>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let items =
        s.us.get_users(
            &pagination,
            &query,
            authenticated.user.as_action_type(false),
            tx.as_mut(),
        )
        .await?;
    let count = s.us.get_users_count(&query, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { items, count }))
}

/// update user by id
#[utoipa::path(
    put,
    path = "/users/{id}",
    responses(
        (status = 200, description = "update user successfully", body = UserInfo)
    ),
    params(
        ("id" = i64, Path, description = "user id")
    )
)]
async fn update_user(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Json(body): Json<UserInfo>,
) -> Result<Json<UserInfo>> {
    authenticated.fail_if_not_admin()?;
    let mut tx = s.ps.begin_tx(true).await?;
    check_user(s.clone(), &body, Some(id), tx.as_mut()).await?;
    if authenticated.user.id != id && body.user_type == UserType::Admin {
        return AppError::custom(
            CustomErrorCode::NoPermission,
            "You don't have to upgrade the user to admin.",
        )
        .into_err();
    }
    let r = s.us.update_user(id, body, tx.as_mut()).await?;
    if let Some(v) = r {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(CustomErrorCode::UserNotFound, "User is not exists.").into_err()
    }
}

/// get user token
#[utoipa::path(
    get,
    path = "/users_token",
    responses(
        (status = 200, description = "get user token successfully", body = AuthenticatedUser)
    ),
    params(
        GetTokenQuery
    )
)]
async fn get_user_token(
    State(s): State<AppState>,
    Query(query): Query<GetTokenQuery>,
    authenticated: Option<AuthenticatedUser>,
) -> Result<Json<AuthenticatedUser>> {
    if authenticated.is_some() {
        return AppError::custom(CustomErrorCode::SameObject, "You have login already.").into_err();
    }
    let mut tx = s.ps.begin_tx(false).await?;
    if let Some(user) =
        s.us.get_user_by_username(&query.username, tx.as_mut())
            .await?
    {
        if user.password == query.password {
            let token = s.us.get_token(&user, tx.as_mut()).await?;
            tx.commit().await?;
            Ok(Json(AuthenticatedUser { user, token }))
        } else {
            AppError::custom(CustomErrorCode::WrongPassword, "Password is wrong.").into_err()
        }
    } else {
        AppError::custom(CustomErrorCode::UserNotFound, "The user not found!").into_err()
    }
}

/// get me by token
#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, description = "get user successfully", body = UserInfo)
    ),
)]
async fn get_me_by_token(
    State(_s): State<AppState>,
    authenticated: Option<AuthenticatedUser>,
) -> Result<Json<AuthenticatedUser>> {
    if let Some(user) = authenticated {
        Ok(Json(user))
    } else {
        AppError::custom(CustomErrorCode::UserNotFound, "Can't found you with token!").into_err()
    }
}

/// remove user token
#[utoipa::path(
    delete,
    path = "/users_token/{id}",
    responses(
        (status = 200, description = "delete user token successfully", body = AuthenticatedUser)
    ),
    params(
        GetTokenQuery,
        ("id" = i64, Path, description = "user id")
    )
)]
async fn remove_user_token(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<StatusCode> {
    if authenticated.user.id != id && authenticated.user.user_type != UserType::Admin {
        return AppError::custom(
            CustomErrorCode::NoPermission,
            "You are not the target user or administrator!",
        )
        .into_err();
    }
    let mut tx = s.ps.begin_tx(true).await?;
    s.us.remove_token(id, tx.as_mut()).await?;
    Ok(StatusCode::OK)
}

/// get user configure
#[utoipa::path(
    get,
    path = "/users_configure/{user_id}",
    responses(
        (status = 200, description = "get user configure successfully", body = UserConfigure)
    ),
    params(
        ("user_id" = i64, Path, description = "user id")
    )
)]
async fn get_user_configure(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<Json<UserConfigure>> {
    if authenticated.user.id != id && authenticated.user.user_type != UserType::Admin {
        return AppError::custom(
            CustomErrorCode::NoPermission,
            "You can't get other user configure",
        )
        .into_err();
    }
    let mut tx = s.ps.begin_tx(false).await?;
    let r = s.us.get_configure(id, tx.as_mut()).await?;
    tx.commit().await?;
    if let Some(p) = r {
        Ok(Json(p))
    } else {
        AppError::custom(CustomErrorCode::UserNotFound, "User is not exists.").into_err()
    }
}

/// update user configure by id
#[utoipa::path(
    put,
    path = "/users_configure/{user_id}",
    responses(
        (status = 200, description = "update user configure successfully", body = UserConfigure)
    ),
    params(
        ("user_id" = i64, Path, description = "user id")
    )
)]
async fn update_user_configure(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Json(body): Json<UserConfigure>,
) -> Result<Json<UserConfigure>> {
    if authenticated.user.id != id && authenticated.user.user_type != UserType::Admin {
        return AppError::custom(
            CustomErrorCode::NoPermission,
            "You can't update other user configure",
        )
        .into_err();
    }
    let mut tx = s.ps.begin_tx(true).await?;
    let r = s.us.update_configure(id, body, tx.as_mut()).await?;
    if let Some(v) = r {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(CustomErrorCode::UserNotFound, "User is not exists.").into_err()
    }
}
