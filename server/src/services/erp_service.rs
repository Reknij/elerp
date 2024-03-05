use crate::{
    custom_error::{AppError, CustomErrorCode},
    erp::{
        area_module::model::{Area, GetAreasQuery}, guest_order_module::model::{GetGuestOrdersQuery, GuestOrder, GuestOrderConfirm, GuestOrderStatus}, inventory_module::model::{GetInventoryQuery, InventoryProduct}, order_module::model::{CheckOrderResult, GetOrdersQuery, Order, OrderItem, OrderType}, order_payment_module::model::{GetOrderPaymentsQuery, OrderPayment}, order_category_module::model::{GetOrderCategoryQuery, OrderCategory}, person_module::model::{GetPersonsQuery, Person}, sku_category_module::model::{GetSKUCategoriesQuery, SKUCategory}, sku_module::model::{GetSKUsQuery, SKU}, statistical_module::model::statistical_data::{GetStatisticalDataQuery, StatisticalData}, warehouse_module::model::{fn_argument::WarehouseIsFrom, warehouse::{GetWarehousesQuery, Warehouse, WarehouseToLinkQuery}}, ActionType
    },
    public_system::model::{ListSlice, Pagination}, user_system::models::user_info::{UserInfo, UserType},
};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde_qs::axum::QsQuery as Query;

use sqlx::SqliteConnection;
use tokio_util::io::ReaderStream;

use utoipa::OpenApi;

use super::{models::{authenticated_user::AuthenticatedUser, clear_result::ClearResult}, AppState};

type Result<T> = core::result::Result<T, AppError>;

#[derive(OpenApi)]
#[openapi(
    paths(
        add_person,
        remove_person,
        get_person,
        get_persons,
        update_person,

        add_area,
        remove_area,
        get_area,
        get_areas,
        update_area,
            
        add_warehouse,
        remove_warehouse,
        get_warehouse,
        get_warehouses,
        update_warehouse,

        inventory_list,
        inventory_list_excel,

        add_order,
        remove_order,
        update_order,
        get_orders,
        get_orders,
        check_order,

        add_order_category,
        remove_order_category,
        update_order,
        get_order_category,
        get_order_categories,

        add_order_payment,
        remove_order_payment,
        get_order_payment,
        get_order_payments,

        add_sku,
        remove_sku,
        get_sku,
        get_skus,
        update_sku,

        add_sku_category,
        remove_sku_category,
        get_sku_category,
        get_sku_categories,
        update_sku_category,
    ),
    tags(
        (name = "ERP", description = "ERP API")
    ),
    components(
        schemas(
            Area,
            Person,
            GetPersonsQuery,
            GetAreasQuery,
            Order,
            OrderType,
            CheckOrderResult,
            Warehouse,
            OrderItem,
            SKU,
            SKUCategory,
            InventoryProduct,
            GetWarehousesQuery,
            GetOrdersQuery,
            GetSKUCategoriesQuery,
            GetSKUsQuery,
        )
    )
)]
pub struct ApiDoc;

pub fn get_services() -> Router<AppState> {
    Router::new()
        .route("/statistical_data", get(get_statistical_data))
        .route("/clear_cache", post(clear_cache))
        .route("/persons", post(add_person).get(get_persons).delete(clear_persons))
        .route(
            "/persons/:id",
            delete(remove_person).get(get_person).put(update_person),
        )
        .route("/areas", post(add_area).get(get_areas).delete(clear_areas))
        .route(
            "/areas/:id",
            delete(remove_area).get(get_area).put(update_area),
        )
        .route("/warehouses", post(add_warehouse).get(get_warehouses).delete(clear_warehouses))
        .route(
            "/warehouses/:id",
            delete(remove_warehouse)
                .get(get_warehouse)
                .put(update_warehouse),
        )
        .route(
            "/warehouse_link/:id",
            delete(unlink_warehouse)
                .get(get_warehouse_linked_users)
                .post(link_warehouse),
        )
        .route("/inventory", get(inventory_list))
        .route("/inventory_excel", get(inventory_list_excel))
        .route("/check_order", post(check_order))
        .route("/orders", post(add_order).get(get_orders).delete(clear_orders))
        .route("/guest_orders", post(add_guest_order).get(get_guest_orders).delete(clear_guest_orders))
        .route("/order_categories", post(add_order_category).get(get_order_categories).delete(clear_order_categories))
        .route("/order_categories/:id", delete(remove_order_category).get(get_order_category).put(update_order_category))
        .route("/order_payments", post(add_order_payment).get(get_order_payments).delete(clear_order_payments))
        .route("/order_payments/:id", delete(remove_order_payment).get(get_order_payment))
        .route("/recalc_orders", post(recalc_orders))
        .route(
            "/orders/:id",
            delete(remove_order).get(get_order).put(update_order),
        )
        .route(
            "/guest_orders/:id",
            delete(remove_guest_order).get(get_guest_order).put(confirm_guest_order),
        )
        .route("/skus", post(add_sku).get(get_skus).delete(clear_skus))
        .route("/skus/:id", delete(remove_sku).get(get_sku).put(update_sku))
        .route(
            "/sku_categories",
            post(add_sku_category).get(get_sku_categories).delete(clear_sku_categories),
        )
        .route(
            "/sku_categories/:id",
            delete(remove_sku_category)
                .get(get_sku_category)
                .put(update_sku_category),
        )
}

/// Get statistics data
#[utoipa::path(
    get,
    path = "/statistical_data",
    responses(
        (status = 200, description = "get statistical data successfully", body = StatisticalData)
    ),
)]
async fn get_statistical_data(State(s): State<AppState>,Query(q): Query<GetStatisticalDataQuery>, authenticated: AuthenticatedUser,) -> Result<Json<StatisticalData>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let data = s.erp.statistical.get(&q, authenticated.user.as_action_type(false), tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(data))
}

/// Clear system cache.
#[utoipa::path(
    post,
    path = "/clear_cache",
    responses(
        (status = 200, description = "clear cache successfully", body = ())
    ),
)]
async fn clear_cache(State(s): State<AppState>, authenticated: AuthenticatedUser,) -> Result<StatusCode> {
    authenticated.fail_if_not_admin()?;
    s.erp.clear_cache().await?;
    s.ps.clear_cache().await?;
    Ok(StatusCode::OK)
}

async fn check_token_exists(s: AppState, authenticated: Option<&AuthenticatedUser>, headers: &HeaderMap, tx: &mut SqliteConnection) -> Result<()> {
    if authenticated.is_none() {
        match headers.get("X-Sub-Authorization") {
            Some(token) => {
                if !s.us.is_sub_token_active(token.to_str().expect("Get sub token str failed!"), &mut *tx).await? {
                    return AppError::custom(CustomErrorCode::NoPermission, "Token expired!").into_err();
                }
            },
            None => return AppError::custom(CustomErrorCode::NoPermission, "Enter your token to continue!").into_err(),
        }
    }
    Ok(())
}

async fn check_person(s: AppState, p: &Person, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<()> {
    if s.erp.person.is_exists_name(&p.name, prev, &mut *tx).await? {
        return AppError::custom(
            CustomErrorCode::SameObject,
            "Already contains the person's name.",
        )
        .into_err();
    }
    if !s.erp.area.is_exists(p.area_id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::AreaNotFound, "Area is not exists.").into_err();
    }
    if p.person_in_charge_id > 0 && !s.erp
            .person
            .is_exists(p.person_in_charge_id, &mut *tx)
            .await?
    {
        return AppError::custom(
            CustomErrorCode::PersonNotFound,
            "Person in charge is not exists.",
        )
        .into_err();
    }

    Ok(())
}

/// Add person
#[utoipa::path(
        post,
        path = "/persons",
        responses(
            (status = 200, description = "add person successfully", body = Person)
        ),
    )]
async fn add_person(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(p): Json<Person>,
) -> Result<Json<Person>> {
    authenticated.is_manage_person()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.erp.person.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::PersonLimitExceeded, "Person count limit exceeded!").into_err();
    }
    check_person(s.clone(), &p, None, tx.as_mut()).await?;
    let r = s.erp.person.add(p, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(r))
}

async fn remove_person_core(s: AppState, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
    if s.erp.person.is_depend_by_another(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::SomeoneIsDepentIt, "Some one is depent to the person.").into_err();
    }
    Ok(s.erp.person.remove(id, notice, &mut *tx).await?)
}

/// remove person
#[utoipa::path(
    delete,
    path = "/persons/{id}",
    responses(
        (status = 200, description = "remove person successfully")
    ),
    params(
        ("id" = i64, Path, description = "person id")
    )
)]
async fn remove_person(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    authenticated.is_manage_person()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if remove_person_core(s.clone(), id, true, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(StatusCode::OK)
    } else {
        AppError::custom(CustomErrorCode::PersonNotFound, "Person is not exists.").into_err()
    }
}

/// clear persons
#[utoipa::path(
    delete,
    path = "/persons",
    responses(
        (status = 200, description = "clear persons successfully")
    ),
    params(
        Pagination,
        GetPersonsQuery,
    )
)]
async fn clear_persons(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Query(q): Query<GetPersonsQuery>,
) -> Result<Json<ClearResult>> {
    authenticated.is_manage_person()?;
    let mut tx = s.ps.begin_tx(true).await?;
    let mut pagination = Pagination::new(0, 100);
    let mut success = 0;
    let mut failed = 0;
    let mut ids = s.erp.person.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    while ids.len() > 0 {
        for id in ids {
            match remove_person_core(s.clone(), id, false, tx.as_mut()).await {
                Ok(_) => success += 1,
                Err(_) => failed += 1,
            }
        }
        pagination.set_offset(failed);
        ids = s.erp.person.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    }
    tx.commit().await?;
    s.ps.notice(crate::public_system::model::WebSocketFlags::ClearPersons).await?;
    Ok(Json(ClearResult {
        success,
        failed,
    }))
}

/// get person
#[utoipa::path(
    get,
    path = "/persons/{id}",
    responses(
        (status = 200, description = "get person successfully", body = Person)
    ),
    params(
        ("id" = i64, Path, description = "person id")
    )
)]
async fn get_person(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<Json<Person>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let action = authenticated.user.as_action_type(authenticated.is_manage_person().is_ok());
    let r = s.erp.person.get(id, action, tx.as_mut()).await?;
    if let Some(p) = r {
        tx.commit().await?;
        Ok(Json(p))
    } else {
        AppError::custom(CustomErrorCode::PersonNotFound, "Person is not exists.").into_err()
    }
}

/// get persons
#[utoipa::path(
    get,
    path = "/persons",
    responses(
        (status = 200, description = "get persons successfully", body = ListSlice<Person>)
    ),
    params(
        Pagination,
        GetPersonsQuery,
    )
)]
async fn get_persons(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(query): Query<GetPersonsQuery>,
    authenticated: AuthenticatedUser,
) -> Result<Json<ListSlice<Person>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let action = authenticated.user.as_action_type(authenticated.is_manage_person().is_ok());
    let items = s.erp.person.get_multiple(&pagination.correct(), &query, action, tx.as_mut()).await?;
    let count = s.erp.person.get_count(&query, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { count, items }))
}

/// update person
#[utoipa::path(
    put,
    path = "/persons/{id}",
    responses(
        (status = 200, description = "update person successfully", body = Person)
    ),
    params(
        ("id" = i64, Path, description = "person id")
    )
)]
async fn update_person(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Json(body): Json<Person>,
) -> Result<Json<Person>> {
    authenticated.is_manage_person()?;
    let mut tx = s.ps.begin_tx(true).await?;
    check_person(s.clone(), &body, Some(id),tx.as_mut()).await?;

    let r = s.erp.person.update(id, body, tx.as_mut()).await?;
    if let Some(v) = r {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(CustomErrorCode::PersonNotFound, "Person is not exists.").into_err()
    }
}

async fn check_area(s: AppState, v: &Area, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<()> {
    if s.erp.area.is_exists_name(&v.name, prev, &mut *tx).await? {
        return AppError::custom(
            CustomErrorCode::SameObject,
            "Already contains the area's name.",
        )
        .into_err();
    }

    Ok(())
}

/// add area
#[utoipa::path(
    post,
    path = "/areas",
    responses(
        (status = 200, description = "add area successfully", body = Area)
    )
)]
async fn add_area(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(area): Json<Area>,
) -> Result<Json<Area>> {
    authenticated.is_manage_area()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.erp.area.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::AreaLimitExceeded, "Area count limit exceeded!").into_err();
    }
    check_area(s.clone(), &area, None, tx.as_mut()).await?;
    let r = s.erp.area.add(area, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(r))
}

async fn remove_area_core(s: AppState, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
    if s.erp.area.is_depend_by_another(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::SomeoneIsDepentIt, "Some one is depent to the area.").into_err();
    }
    Ok(s.erp.area.remove(id, notice, &mut *tx).await?)
}

/// remove area
#[utoipa::path(
    delete,
    path = "/areas/{id}",
    responses(
        (status = 200, description = "remove area successfully")
    ),
    params(
        ("id" = i64, Path, description = "area id")
    )
)]
async fn remove_area(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<StatusCode> {
    authenticated.is_manage_area()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if remove_area_core(s.clone(), id, true, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(StatusCode::OK)
    } else {
        AppError::custom(CustomErrorCode::AreaNotFound, "Area is not exists.").into_err()
    }
}

/// clear areas
#[utoipa::path(
    delete,
    path = "/areas",
    responses(
        (status = 200, description = "clear areas successfully")
    ),
    params(
        Pagination,
        GetAreasQuery,
    )
)]
async fn clear_areas(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Query(q): Query<GetAreasQuery>,
) -> Result<Json<ClearResult>> {
    authenticated.is_manage_area()?;
    let mut tx = s.ps.begin_tx(true).await?;
    let mut pagination = Pagination::new(0, 100);
    let mut success = 0;
    let mut failed = 0;
    let mut ids = s.erp.area.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    while ids.len() > 0 {
        for id in ids {
            match remove_area_core(s.clone(), id, false, tx.as_mut()).await {
                Ok(_) => success += 1,
                Err(_) => failed += 1,
            }
        }
        pagination.set_offset(failed);
        ids = s.erp.area.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    }
    tx.commit().await?;
    s.ps.notice(crate::public_system::model::WebSocketFlags::ClearAreas).await?;
    Ok(Json(ClearResult {
        success,
        failed,
    }))
}

/// get area
#[utoipa::path(
    get,
    path = "/areas/{id}",
    responses(
        (status = 200, description = "get area successfully", body =Area)
    ),
    params(
        ("id" = i64, Path, description = "area id")
    )
)]
async fn get_area(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    _: AuthenticatedUser,
) -> Result<Json<Area>> {
    let mut tx = s.ps.begin_tx(false).await?;
    if !s.erp.area.is_exists(id, tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::AreaNotFound, "Area is not exists.").into_err();
    }
    Ok(Json(s.erp.area.get(id, tx.as_mut()).await?.unwrap()))
}

/// get areas
#[utoipa::path(
    get,
    path = "/areas",
    responses(
        (status = 200, description = "get areas successfully", body = [Area])
    ),
    params(
        Pagination,
        GetAreasQuery
    ),
)]
async fn get_areas(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(query): Query<GetAreasQuery>,
    _: AuthenticatedUser,
) -> Result<Json<ListSlice<Area>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let items = s.erp.area.get_multiple(&pagination.correct(), &query, tx.as_mut()).await?;
    let count = s.erp.area.get_count(&query, &mut *tx).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { count, items }))
}

/// update area
#[utoipa::path(
    put,
    path = "/areas/{id}",
    responses(
        (status = 200, description = "update area successfully", body = Area)
    ),
    params(
        ("id" = i64, Path, description = "area id")
    )
)]
async fn update_area(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Json(body): Json<Area>,
) -> Result<Json<Area>> {
    authenticated.is_manage_area()?;
    let mut tx = s.ps.begin_tx(true).await?;
    check_area(s.clone(), &body, Some(id), tx.as_mut()).await?;
    let r = s.erp.area.update(id, body, tx.as_mut()).await?;
    if let Some(v) = r {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(CustomErrorCode::AreaNotFound, "Area is not exists.").into_err()
    }
}

async fn check_warehouse(s: AppState, v: &Warehouse, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<()> {
    if s.erp.warehouse.is_exists_name(&v.name, prev, &mut *tx).await? {
        return AppError::custom(
            CustomErrorCode::SameObject,
            "Already contains the warehouse's name.",
        )
        .into_err();
    }
    if !s.erp.person.is_exists(v.person_in_charge_id, &mut *tx).await? {
        return AppError::custom(
            CustomErrorCode::PersonNotFound,
            "Person in charge is not exists.",
        )
        .into_err();
    }
    if !s.erp.area.is_exists(v.area_id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::AreaNotFound, "Area is not exists.").into_err();
    }
    Ok(())
}

/// add warehouse
#[utoipa::path(
    post,
    path = "/warehouses",
    responses(
        (status = 200, description = "add warehouse successfully", body = Warehouse)
    ),
)]
async fn add_warehouse(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(body): Json<Warehouse>,
) -> Result<Json<Warehouse>> {
    authenticated.is_manage_warehouse()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.erp.warehouse.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::WarehouseLimitExceeded, "Warehouse count limit exceeded!").into_err();
    }
    check_warehouse(s.clone(), &body, None, tx.as_mut()).await?;
    let v = s.erp.warehouse.add(body, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(v))
}

async fn remove_warehouse_core(s: AppState, id: i64, notice: bool, user: &UserInfo, tx: &mut SqliteConnection) -> Result<bool> {
    if s.erp.warehouse.is_depend_by_another(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::SomeoneIsDepentIt, "Some one is depent to the warehouse.").into_err();
    }
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::ID(id), user.into(), &mut *tx).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to the warehouse!").into_err();
    }

    Ok(s.erp.warehouse.remove(id, notice, &mut *tx).await?)
}
/// remove warehouse
#[utoipa::path(
    delete,
    path = "/warehouses/{id}",
    responses(
        (status = 200, description = "remove warehouse successfully")
    ),
    params(
        ("id" = i64, Path, description = "warehouse id")
    )
)]
async fn remove_warehouse(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<StatusCode> {
    authenticated.is_manage_warehouse()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if remove_warehouse_core(s.clone(), id, true, &authenticated.user, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(StatusCode::OK)
    } else {
        AppError::custom(
            CustomErrorCode::WarehouseNotFound,
            "Warehouse is not exists.",
        )
        .into_err()
    }
}

/// clear warehouses
#[utoipa::path(
    delete,
    path = "/warehouses",
    responses(
        (status = 200, description = "clear warehouses successfully")
    ),
    params(
        Pagination,
        GetWarehousesQuery,
    )
)]
async fn clear_warehouses(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Query(q): Query<GetWarehousesQuery>,
) -> Result<Json<ClearResult>> {
    authenticated.is_manage_warehouse()?;
    let mut tx = s.ps.begin_tx(true).await?;
    let mut pagination = Pagination::new(0, 100);
    let action = authenticated.user.as_action_type(true);
    let mut success = 0;
    let mut failed = 0;
    let mut ids = s.erp.warehouse.get_multiple_ids(&pagination, &q, action, tx.as_mut()).await?;
    while ids.len() > 0 {
        for id in ids {
            match remove_warehouse_core(s.clone(), id, false, &authenticated.user, tx.as_mut()).await {
                Ok(_) => success += 1,
                Err(_) => failed += 1,
            }
        }
        pagination.set_offset(failed);
        ids = s.erp.warehouse.get_multiple_ids(&pagination, &q, action, tx.as_mut()).await?;
    }
    tx.commit().await?;
    s.ps.notice(crate::public_system::model::WebSocketFlags::ClearWarehouses).await?;
    Ok(Json(ClearResult {
        success,
        failed,
    }))
}

/// get warehouse by id
#[utoipa::path(
    get,
    path = "/warehouses/{id}",
    responses(
        (status = 200, description = "get warehouse successfully", body = Warehouse)
    ),
    params(
        ("id" = i64, Path, description = "warehouse id")
    )
)]
async fn get_warehouse(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<Json<Warehouse>> {
    let mut tx = s.ps.begin_tx(false).await?;
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::ID(id), (&authenticated.user).into(), tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to the warehouse!").into_err();
    }
    if let Some(p) = s.erp.warehouse.get(id, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(Json(p))
    } else {
        AppError::custom(
            CustomErrorCode::WarehouseNotFound,
            "Warehouse is not exists.",
        )
        .into_err()
    }
}

/// get warehouses
#[utoipa::path(
    get,
    path = "/warehouses",
    responses(
        (status = 200, description = "get warehouse successfully", body = ListSlice<Warehouse>)
    ),
    params(
        Pagination,
        GetWarehousesQuery,
    )
)]
async fn get_warehouses(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(query): Query<GetWarehousesQuery>,
    authenticated: AuthenticatedUser,
) -> Result<Json<ListSlice<Warehouse>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let action = authenticated.user.as_action_type(authenticated.is_manage_warehouse().is_ok());
    let items = s.erp.warehouse.get_multiple(&pagination.correct(), &query, action, tx.as_mut()).await?;
    let count = s.erp.warehouse.get_count(&query, action, &mut *tx).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { items, count }))
}

/// update warehouse by id
#[utoipa::path(
    put,
    path = "/warehouses/{id}",
    responses(
        (status = 200, description = "update warehouse successfully", body = Warehouse)
    ),
    params(
        ("id" = i64, Path, description = "warehouse id")
    )
)]
async fn update_warehouse(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,

    Json(body): Json<Warehouse>,
) -> Result<Json<Warehouse>> {
    authenticated.is_manage_warehouse()?;
    let mut tx = s.ps.begin_tx(true).await?;
    check_warehouse(s.clone(), &body, Some(id), tx.as_mut()).await?;
    if let Some(v) = s.erp.warehouse.update(id, body, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(
            CustomErrorCode::WarehouseNotFound,
            "Warehouse is not exists.",
        )
        .into_err()
    }
}

async fn check_link_warehouse(s: AppState, user: &UserInfo, warehouse_id: i64, q: &WarehouseToLinkQuery, to_link: bool, tx: &mut SqliteConnection) -> Result<()> {
    if user.user_type == UserType::Admin && user.id == q.user_id {
        return AppError::custom(CustomErrorCode::NotAllowed, "Admin is not required to linked warehouses.").into_err();
    }
    if !s.erp.warehouse.is_exists(warehouse_id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::WarehouseNotFound, "Warehouse is not found!").into_err();
    }
    if !s.us.is_exists(q.user_id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::UserNotFound, "User is not found!").into_err();
    }
    if to_link == s.erp.warehouse.is_linked(WarehouseIsFrom::ID(warehouse_id), q.user_id.into(), &mut *tx).await? {
        return if to_link {
            AppError::custom(CustomErrorCode::Linked, "Warehouse is linked user already!").into_err()
        } else {
            AppError::custom(CustomErrorCode::Unlinked, "Warehouse is unlinked user already!").into_err()
        }
    }
    Ok(())
}

/// link warehouse
#[utoipa::path(
    post,
    path = "/warehouse_link/{id}",
    responses(
        (status = 200, description = "linked warehouse successfully", body = StatusCode)
    ),
)]
async fn link_warehouse(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Query(q): Query<WarehouseToLinkQuery>,
) -> Result<StatusCode> {
    authenticated.fail_if_not_admin()?;
    let mut tx = s.ps.begin_tx(true).await?;
    check_link_warehouse(s.clone(), &authenticated.user, id, &q, true, tx.as_mut()).await?;
    Ok(if s.erp.warehouse.link(id, q.user_id, tx.as_mut()).await? {
        tx.commit().await?;
        StatusCode::OK
    } else {
        StatusCode::EXPECTATION_FAILED
    })
}

/// unlink warehouse
#[utoipa::path(
    delete,
    path = "/warehouse_link/{id}",
    responses(
        (status = 200, description = "unlinked warehouse successfully", body = StatusCode)
    ),
)]
async fn unlink_warehouse(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Query(q): Query<WarehouseToLinkQuery>,
) -> Result<StatusCode> {
    authenticated.fail_if_not_admin()?;
    let mut tx = s.ps.begin_tx(true).await?;
    check_link_warehouse(s.clone(), &authenticated.user, id, &q, false, tx.as_mut()).await?;
    Ok(if s.erp.warehouse.unlink(id, q.user_id, tx.as_mut()).await? {
        tx.commit().await?;
        StatusCode::OK
    } else {
        StatusCode::EXPECTATION_FAILED
    })
}

/// get warehouse linked users
#[utoipa::path(
    get,
    path = "/warehouse_link/{id}",
    responses(
        (status = 200, description = "get warehouse linked users successfully", body = ListSlice<UserInfo>)
    ),
)]
async fn get_warehouse_linked_users(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Query(pagination): Query<Pagination>,
    authenticated: AuthenticatedUser,
) -> Result<Json<ListSlice<UserInfo>>> {
    authenticated.fail_if_not_admin()?;
    let mut tx = s.ps.begin_tx(false).await?;
    let items = s.erp.warehouse.get_linked_users(id, &pagination, tx.as_mut()).await?;
    let count = s.erp.warehouse.get_linked_users_count(id, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { count, items }))
}

async fn check_order_category(s: AppState, v: &OrderCategory, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<()> {
    if s.erp.order_category.is_exists_name(&v.name, prev, &mut *tx).await? {
        return AppError::custom(
            CustomErrorCode::SameObject,
            "Already contains the order category's name.",
        )
        .into_err();
    }
    Ok(())
}
/// add order category
#[utoipa::path(
    post,
    path = "/order_categories",
    responses(
        (status = 200, description = "add order category successfully", body = OrderCategory)
    ),
)]
async fn add_order_category(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(body): Json<OrderCategory>,
) -> Result<Json<OrderCategory>> {
    authenticated.is_manage_order_category()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.erp.order_category.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::OrderCategoryLimitExceeded, "Order category count limit exceeded!").into_err();
    }
    check_order_category(s.clone(), &body, None, tx.as_mut()).await?;
    let r = s.erp.order_category.add(body, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(r))
}

async fn remove_order_category_core(s: AppState, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
    if id == 10001 {
        return AppError::custom(CustomErrorCode::NotAllowed, "Order category must contain first one category!").into_err();
    }
    if s.erp.order_category.is_depend_by_another(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::SomeoneIsDepentIt, "Some one is depent to the order category.").into_err();
    }
    Ok(s.erp.order_category.remove(id, notice, &mut *tx).await?)
}

/// remove order category
#[utoipa::path(
    delete,
    path = "/order_categories/{id}",
    responses(
        (status = 200, description = "remove order category successfully")
    ),
    params(
        ("id"=i64, Path, description = "order category id")
    )
)]
async fn remove_order_category(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    authenticated.is_manage_order_category()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if remove_order_category_core(s.clone(), id, true, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(StatusCode::OK)
    } else {
        AppError::custom(CustomErrorCode::OrderCategoryNotFound, "Order status is not found.").into_err()
    }
}

/// clear order category
#[utoipa::path(
    delete,
    path = "/order_categories",
    responses(
        (status = 200, description = "clear order categories successfully")
    ),
    params(
        Pagination,
        GetOrderCategoryQuery,
    )
)]
async fn clear_order_categories(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Query(q): Query<GetOrderCategoryQuery>,
) -> Result<Json<ClearResult>> {
    authenticated.is_manage_order_category()?;
    let mut tx = s.ps.begin_tx(true).await?;
    let mut pagination = Pagination::new(0, 100);
    let mut success = 0;
    let mut failed = 0;
    let mut ids = s.erp.order_category.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    while ids.len() > 0 {
        for id in ids {
            match remove_order_category_core(s.clone(), id, false, tx.as_mut()).await {
                Ok(_) => success += 1,
                Err(_) => failed += 1,
            }
        }
        pagination.set_offset(failed);
        ids = s.erp.order_category.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    }
    tx.commit().await?;
    s.ps.notice(crate::public_system::model::WebSocketFlags::ClearOrderCategories).await?;
    Ok(Json(ClearResult {
        success,
        failed,
    }))
}

/// get order category
#[utoipa::path(
    get,
    path = "/order_categories/{id}",
    responses(
        (status = 200, description = "get order category successfully", body = OrderCategory)
    ),
    params(
        ("id"=i64, Path, description = "order category id")
    )
)]
async fn get_order_category(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    _: AuthenticatedUser,
) -> Result<Json<OrderCategory>> {
    let mut tx = s.ps.begin_tx(false).await?;
    if let Some(v) = s.erp.order_category.get(id, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(
            CustomErrorCode::OrderCategoryNotFound,
            "Order status is not found.",
        )
        .into_err()
    }
}

/// get order category list
#[utoipa::path(
    get,
    path = "/order_categories",
    responses(
        (status = 200, description = "get order_categories successfully", body = ListSlice<OrderCategory>)
    ),
    params(
        Pagination,
        GetOrderCategoryQuery
    )
)]
async fn get_order_categories(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(q): Query<GetOrderCategoryQuery>,
    _: AuthenticatedUser,
) -> Result<Json<ListSlice<OrderCategory>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let items = s.erp.order_category.get_multiple(&pagination.correct(), &q, &mut *tx).await?;
    let count = s.erp.order_category.get_count(&q, &mut *tx).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { items, count }))
}

/// update order category
#[utoipa::path(
    put,
    path = "/order_categories/{id}",
    responses(
        (status = 200, description = "update order category successfully", body = OrderCategory)
    ),
    params(
        ("id"=i64, Path, description = "order category id")
    )
)]
async fn update_order_category(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Json(body): Json<OrderCategory>,
) -> Result<Json<OrderCategory>> {
    authenticated.is_manage_order_category()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if let Some(v) = s.erp.order_category.update(id, body, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(
            CustomErrorCode::OrderCategoryNotFound,
            "Order status is not found.",
        )
        .into_err()
    }
}

async fn check_order_payment(s: AppState, user: &UserInfo, op: &OrderPayment, tx: &mut SqliteConnection) -> Result<()> {
    if !s.erp.order.is_exists(op.order_id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::OrderNotFound, "Order is not found.").into_err();
    }
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::ID(op.warehouse_id), user.into(), &mut *tx).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to warehouse!").into_err();
    }
    if !s.erp.person.is_exists(op.person_in_charge_id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::PersonNotFound, "The person in charge is not found!").into_err();
    }
    let status = s.erp.order.get_order_payment_status(op.order_id, tx).await?.unwrap();
    match status {
        crate::erp::order_module::model::OrderPaymentStatus::Settled => return AppError::custom(
            CustomErrorCode::OrderPaymentSettled,
            "Order payment is settled.",
        ).into_err(),
        crate::erp::order_module::model::OrderPaymentStatus::Unsettled |
        crate::erp::order_module::model::OrderPaymentStatus::PartialSettled => (),
        crate::erp::order_module::model::OrderPaymentStatus::None => return AppError::custom(
            CustomErrorCode::OrderPaymentIsNone,
            "Order payment is none.",
        ).into_err(),
    }
    if op.total_amount <= 0.0 {
        return AppError::custom(CustomErrorCode::TotalAmountUnexpected, "Total amount is unexpected.").into_err();
    }
    Ok(())
}

/// add order payment
#[utoipa::path(
    post,
    path = "/order_payments",
    responses(
        (status = 200, description = "add order payment successfully", body = OrderPayment)
    ),
)]
async fn add_order_payment(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(mut body): Json<OrderPayment>,
) -> Result<Json<OrderPayment>> {
    authenticated.is_add_order_payment()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.erp.order_payment.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::OrderPaymentLimitExceeded, "Order payment count limit exceeded!").into_err();
    }
    s.erp.order_payment.preprocess(&mut body, &authenticated.user);
    check_order_payment(s.clone(), &authenticated.user, &body, tx.as_mut()).await?;
    let r = s.erp.order_payment.add(body, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(r))
}

async fn remove_order_payment_core(s: AppState, user: &UserInfo, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
    if !s.erp.order_payment.can_access(id, &user, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::NoPermission, "You not the owner or admin!").into_err();
    }
    if !s.erp.order_payment.is_exists(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::OrderPaymentNotFound, "Order payment is not found.").into_err();
    }
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::OrderPayment(id), user.into(), &mut *tx).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to warehouse!").into_err();
    }
    Ok(s.erp.order_payment.remove(id, notice, &mut *tx).await?)
}

/// remove order payment
#[utoipa::path(
    delete,
    path = "/order_payments/{id}",
    responses(
        (status = 200, description = "remove order payment successfully")
    ),
    params(
        ("id"=i64, Path, description = "order payment id")
    )
)]
async fn remove_order_payment(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    authenticated.is_update_remove_order_payment()?;
    let mut tx = s.ps.begin_tx(true).await?;
    remove_order_payment_core(s.clone(), &authenticated.user, id, true, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(StatusCode::OK)
}

/// clear order payments
#[utoipa::path(
    delete,
    path = "/order_payments",
    responses(
        (status = 200, description = "clear order payments successfully")
    ),
    params(
        Pagination,
        GetOrderPaymentsQuery,
    )
)]
async fn clear_order_payments(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Query(q): Query<GetOrderPaymentsQuery>,
) -> Result<Json<ClearResult>> {
    authenticated.is_update_remove_order_payment()?;
    let mut tx = s.ps.begin_tx(true).await?;
    let action = authenticated.user.as_action_type(false);
    let mut pagination = Pagination::new(0, 100);
    let mut success = 0;
    let mut failed = 0;
    let mut ids = s.erp.order_payment.get_multiple_ids(&pagination, &q, action, tx.as_mut()).await?;
    while ids.len() > 0 {
        for id in ids {
            match remove_order_payment_core(s.clone(), &authenticated.user, id, false, tx.as_mut()).await {
                Ok(_) => success += 1,
                Err(_) => failed += 1,
            }
        }
        pagination.set_offset(failed);
        ids = s.erp.order_payment.get_multiple_ids(&pagination, &q, action, tx.as_mut()).await?;
    }
    tx.commit().await?;
    s.ps.notice(crate::public_system::model::WebSocketFlags::ClearOrderPayments).await?;
    Ok(Json(ClearResult {
        success,
        failed,
    }))
}

/// get order payment
#[utoipa::path(
    get,
    path = "/order_payments/{id}",
    responses(
        (status = 200, description = "get order payment successfully", body = OrderPayment)
    ),
    params(
        ("id"=i64, Path, description = "order payment id")
    )
)]
async fn get_order_payment(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<Json<OrderPayment>> {
    let mut tx = s.ps.begin_tx(false).await?;
    if !s.erp.order_payment.is_exists(id, tx.as_mut()).await? {
        return AppError::custom(
            CustomErrorCode::OrderPaymentNotFound,
            "Order payment is not found.",
        )
        .into_err();
    }
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::OrderPayment(id), (&authenticated.user).into(), tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to warehouse!").into_err();
    }
   Ok(Json(s.erp.order_payment.get(id, tx.as_mut()).await?.unwrap()))
}

/// get order payment list
#[utoipa::path(
    get,
    path = "/order_payments",
    responses(
        (status = 200, description = "get order_payments successfully", body = ListSlice<OrderPayment>)
    ),
    params(
        Pagination,
        GetOrderPaymentsQuery
    )
)]
async fn get_order_payments(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(q): Query<GetOrderPaymentsQuery>,
    authenticated: AuthenticatedUser,
) -> Result<Json<ListSlice<OrderPayment>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let action = authenticated.user.as_action_type(false);
    let items = s.erp.order_payment.get_multiple(&pagination.correct(), &q, action, &mut *tx).await?;
    let count = s.erp.order_payment.get_count(&q, action, &mut *tx).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { items, count }))
}

/// check order
#[utoipa::path(
    post,
    path = "/check_order",
    responses(
        (status = 200, description = "get check order result successfully", body = CheckOrderResult)
    ),
)]
async fn check_order(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(mut order): Json<Order>,
) -> Result<Json<CheckOrderResult>> {
    let mut tx = s.ps.begin_tx(false).await?;
    check_order_and_preprocess(s.clone(), &authenticated, true, &mut order, tx.as_mut()).await?;
    let r = s.erp.order.check(&order, false, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(r))
}

async fn check_order_and_preprocess(s: AppState, authenticated: &AuthenticatedUser, initial: bool, order: &mut Order, tx: &mut SqliteConnection) -> Result<()> {
    if initial {
        if !s.erp.warehouse.is_exists(order.warehouse_id, &mut *tx).await? {
            return AppError::custom(
                CustomErrorCode::WarehouseNotFound,
                "Warehouse is not found.",
            )
            .into_err();
        }
        if !s.erp.warehouse.is_linked(WarehouseIsFrom::ID(order.warehouse_id), (&authenticated.user).into(), &mut *tx).await? {
            return AppError::custom(CustomErrorCode::NotLinked, "You not linked to warehouse!").into_err();
        }
    } else {
        if !s.erp.order_category.is_exists(order.order_category_id, tx).await? {
            return AppError::custom(CustomErrorCode::OrderCategoryNotFound, "Order category is not found.").into_err();
        }
    }
    if let Some(person) = s.erp.person.get(order.person_related_id, ActionType::System, &mut *tx).await? {
        s.erp.order.preprocess(order, &authenticated.user, initial, person.person_in_charge_id);
        if order.items.len() == 0 {
            return AppError::custom(CustomErrorCode::OrderItemsIsEmpty, "Items is empty!").into_err();
        }
    } else {
        return AppError::custom(CustomErrorCode::PersonNotFound, "Person is not found.").into_err();
    }
    Ok(())
}

async fn check_guest_order_and_preprocess(s: AppState, authenticated: &AuthenticatedUser, order: &mut GuestOrder, tx: &mut SqliteConnection) -> Result<()> {
    if !s.erp.warehouse.is_exists(order.warehouse_id, &mut *tx).await? {
        return AppError::custom(
            CustomErrorCode::WarehouseNotFound,
            "Warehouse is not found.",
        )
        .into_err();
    }
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::ID(order.warehouse_id), (&authenticated.user).into(), &mut *tx).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to warehouse!").into_err();
    }
    if !s.erp.order_category.is_exists(order.order_category_id, tx).await? {
        return AppError::custom(CustomErrorCode::OrderCategoryNotFound, "Order category is not found.").into_err();
    }
    if let Some(person) = s.erp.person.get(order.person_related_id, ActionType::System, &mut *tx).await? {
        s.erp.guest_order.preprocess(order, &authenticated.user, person.person_in_charge_id);
    } else {
        return AppError::custom(CustomErrorCode::PersonNotFound, "Person is not found.").into_err();
    }
    Ok(())
}

/// add order
#[utoipa::path(
    post,
    path = "/orders",
    responses(
        (status = 200, description = "add order successfully", body = Order)
    ),
)]
async fn add_order(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(mut order): Json<Order>,
) -> Result<Json<Order>> {
    authenticated.is_add_order()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.erp.order.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::OrderLimitExceeded, "Order count limit exceeded!").into_err();
    }
    check_order_and_preprocess(s.clone(), &authenticated, true, &mut order, tx.as_mut()).await?;
    if !s.erp.order.is_check_pass(&order, tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::CheckFailed, "Order can't pass the check!")
            .into_err();
    }
    let r = s.erp.order.add(order, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(r))
}

/// add guest order
#[utoipa::path(
    post,
    path = "/guest_orders",
    responses(
        (status = 200, description = "add guest order successfully", body = Order)
    ),
)]
async fn add_guest_order(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(mut order): Json<GuestOrder>,
) -> Result<Json<GuestOrder>> {
    authenticated.is_add_order()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.erp.guest_order.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::GuestOrderLimitExceeded, "Guest order count limit exceeded!").into_err();
    }
    check_guest_order_and_preprocess(s.clone(), &authenticated, &mut order, tx.as_mut()).await?;
    let sub_token = s.us.get_sub_token(&authenticated.user, tx.as_mut()).await?;
    let r = s.erp.guest_order.add(&sub_token, order, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(r))
}

/// confirm guest order
#[utoipa::path(
    put,
    path = "/guest_orders/{id}",
    responses(
        (status = 200, description = "confirm guest order successfully", body = Order)
    ),
    params(
        ("id"=i64, Path, description = "order id")
    )
)]
async fn confirm_guest_order(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<GuestOrder>,
) -> Result<Json<GuestOrderConfirm>> {
    let mut tx = s.ps.begin_tx(true).await?;
   
    if let Some(mut guest_order) = s.erp.guest_order.get(id, tx.as_mut()).await? {
        match headers.get("X-Sub-Authorization") {
            Some(client_token) => {
              if client_token != &guest_order.sub_token {
                  return AppError::custom(CustomErrorCode::NoPermission, "You no permission to confirm!").into_err();
               }
            },
            None => {
                return AppError::custom(CustomErrorCode::NoPermission, "You no permission to confirm!").into_err();
            }
          };
        let user = match s.us.get_sub_token_owner(&guest_order.sub_token, tx.as_mut()).await? {
            Some(v) => v,
            None => {
                return AppError::custom(CustomErrorCode::UserNotFound, "Can't found the owner of your token!").into_err();
            }
        };
        match guest_order.guest_order_status {
            GuestOrderStatus::Confirmed => {
                return AppError::custom(CustomErrorCode::GuestOrderConfirmed, "Guest order is confirmed already!").into_err();
            },
            GuestOrderStatus::Expired => {
                return AppError::custom(CustomErrorCode::GuestOrderExpired, "Guest order is expired already!").into_err();
            },
            GuestOrderStatus::Pending => ()
        };
        
        guest_order.description = body.description;
        guest_order.items = body.items;
        
        let mut order: Order = guest_order.into();
        let pici = order.person_in_charge_id;
        s.erp.order.preprocess(&mut order, &user, true, pici);
        
        if order.items.len() == 0 {
            return AppError::custom(CustomErrorCode::OrderItemsIsEmpty, "Order items is empty!").into_err();
        }

        let mut result = GuestOrderConfirm {
            check_result: s.erp.order.check(&order, false, tx.as_mut()).await?,
            order: None,
        };
        if result.check_result.items_not_available.len() == 0 {
            order.from_guest_order_id = id;
            let order = s.erp.order.add(order, tx.as_mut()).await?;
            s.erp.guest_order.confirm(id, &order, tx.as_mut()).await?;
            let go = s.erp.guest_order.get(id, tx.as_mut()).await?.expect("Get empty guest order when confirm!");
            tx.commit().await?;
            result.order = Some(go);
        }
       Ok(Json(result))
    } else {
        AppError::custom(CustomErrorCode::OrderNotFound, "Order is not found.").into_err()
    }
   
}

async fn remove_order_core(s: AppState, user: &UserInfo, id: i64, recall: bool, notice: bool, tx: &mut SqliteConnection)-> Result<bool> {
    let action = user.as_action_type(false);

    if !s.erp.order.is_exists(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::OrderNotFound, "Order is not found.").into_err();
    }
    if s.erp.order.is_from_guest_order(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::FromGuestOrder, "This order is created from guest order!").into_err();
    }
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::Order(id), user.into(), &mut *tx).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to warehouse!").into_err();
    }
    if !s.erp.order.can_access(id, &user, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::NoPermission, "You not the order's owner or admin!").into_err();
    }
    if s.erp.order.is_depend_by_another(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::SomeoneIsDepentIt, "Some one is depent to the order.").into_err();
    }
    Ok(s.erp.order.remove(id, recall, notice, action, &mut *tx).await?)
}

/// remove order
#[utoipa::path(
    delete,
    path = "/orders/{id}",
    responses(
        (status = 200, description = "remove order successfully")
    ),
    params(
        ("id"=i64, Path, description = "order id")
    )
)]
async fn remove_order(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    authenticated.is_update_remove_order()?;
    let mut tx = s.ps.begin_tx(true).await?;
    remove_order_core(s.clone(), &authenticated.user, id, true, true, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(StatusCode::OK)
}

/// clear orders
#[utoipa::path(
    delete,
    path = "/orders",
    responses(
        (status = 200, description = "clear orders successfully")
    ),
    params(
        Pagination,
        GetOrdersQuery,
    )
)]
async fn clear_orders(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Query(q): Query<GetOrdersQuery>,
) -> Result<Json<ClearResult>> {
    authenticated.is_update_remove_order()?;
    let mut tx = s.ps.begin_tx(true).await?;
    let action = authenticated.user.as_action_type(false);
    let mut pagination = Pagination::new(0, 100);
    let mut success = 0;
    let mut failed = 0;
    let mut ids = s.erp.order.get_multiple_ids(&pagination, &q, action, tx.as_mut()).await?;
    while ids.len() > 0 {
        for id in ids {
            match remove_order_core(s.clone(), &authenticated.user, id, false, false, tx.as_mut()).await {
                Ok(_) => success += 1,
                Err(_) => failed += 1,
            }
        }
        pagination.set_offset(failed);
        ids = s.erp.order.get_multiple_ids(&pagination, &q, action, tx.as_mut()).await?;
    }
    if success > 0 {
        s.erp.order.recalc_all(None, None, None, action, tx.as_mut()).await?;
    }
    tx.commit().await?;
    s.ps.notice(crate::public_system::model::WebSocketFlags::ClearOrders).await?;
    Ok(Json(ClearResult {
        success,
        failed,
    }))
}

async fn remove_guest_order_core(s: AppState, user: &UserInfo, id: i64, recall: bool, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
    let action = user.as_action_type(false);
    if !s.erp.guest_order.is_exists(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::OrderNotFound, "Guest's order is not found.").into_err()
    }
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::GuestOrder(id), (user).into(), &mut *tx).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to warehouse!").into_err();
    }
    if !s.erp.guest_order.can_access(id, user, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::NoPermission, "You not the order's owner or admin!").into_err();
    }
    let order_id = s.erp.guest_order.get_order_id(id, &mut *tx).await?.unwrap();
    if order_id > 0 {
        if s.erp.order.is_depend_by_another(order_id, &mut *tx).await? {
            return AppError::custom(CustomErrorCode::SomeoneIsDepentIt, "The guest order has the order id is depent to the order.").into_err();
        }
    }
    
    let r = s.erp.guest_order.remove(id, notice, &mut *tx).await?;
        if order_id>0 {
            s.erp.order.remove(order_id, recall,notice,  action, &mut *tx).await?;
        }
        Ok(r)
}

/// remove guest order
#[utoipa::path(
    delete,
    path = "/guest_orders/{id}",
    responses(
        (status = 200, description = "remove guest order successfully")
    ),
    params(
        ("id"=i64, Path, description = "guest order id")
    )
)]
async fn remove_guest_order(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    authenticated.is_update_remove_order()?;
    let mut tx = s.ps.begin_tx(true).await?;
    remove_guest_order_core(s.clone(), &authenticated.user, id, true, true, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(StatusCode::OK)
}

/// clear guest orders
#[utoipa::path(
    delete,
    path = "/guest_orders",
    responses(
        (status = 200, description = "clear orders successfully")
    ),
    params(
        Pagination,
        GetGuestOrdersQuery,
    )
)]
async fn clear_guest_orders(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Query(q): Query<GetGuestOrdersQuery>,
) -> Result<Json<ClearResult>> {
    authenticated.is_update_remove_order()?;
    let mut tx = s.ps.begin_tx(false).await?;
    let action = authenticated.user.as_action_type(false);
    let mut pagination = Pagination::new(0, 100);
    let mut success = 0;
    let mut failed = 0;
    let mut ids = s.erp.guest_order.get_multiple_ids(&pagination, &q, action, tx.as_mut()).await?;
    while ids.len() > 0 {
        for guest_id in ids {
            match remove_guest_order_core(s.clone(), &authenticated.user, guest_id, false, false, tx.as_mut()).await {
                Ok(_) => success += 1,
                Err(_) => failed += 1,
            }
        }
        pagination.set_offset(failed);
        ids = s.erp.guest_order.get_multiple_ids(&pagination, &q, action, tx.as_mut()).await?;
    }
    if success > 0 {
        s.erp.order.recalc_all(None, None, None, action, tx.as_mut()).await?;
    }
    tx.commit().await?;
    s.ps.notice(crate::public_system::model::WebSocketFlags::ClearOrders).await?;
    Ok(Json(ClearResult {
        success,
        failed,
    }))
}

/// recalc all order
#[utoipa::path(
    post,
    path = "/recalc_orders",
    responses(
        (status = 200, description = "recalc all order successfully")
    ),
)]
async fn recalc_orders(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
) -> Result<StatusCode> {
    authenticated.fail_if_not_admin()?;
    let mut tx = s.ps.begin_tx(true).await?;
    let action = authenticated.user.as_action_type(false);
    s.erp.order.recalc_all(None, None, None, action, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(StatusCode::OK)
}

/// get order
#[utoipa::path(
    get,
    path = "/orders/:id",
    responses(
        (status = 200, description = "get order successfully", body = Order)
    ),
    params(
        ("id"=i64, Path, description = "order id")
    )
)]
async fn get_order(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<Json<Order>> {
    let mut tx = s.ps.begin_tx(false).await?;
    if !s.erp.order.is_exists(id, tx.as_mut()).await? {
        return AppError::custom(
            CustomErrorCode::OrderNotFound,
            "Order is not found.",
        )
        .into_err();
    }
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::Order(id), (&authenticated.user).into(), tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to warehouse!").into_err();
    }
    Ok(Json(s.erp.order.get(id, tx.as_mut()).await?.unwrap()))
}

/// get guest order
#[utoipa::path(
    get,
    path = "/guest orders/:id",
    responses(
        (status = 200, description = "get guest order successfully", body = GuestOrder)
    ),
    params(
        ("id"=i64, Path, description = "order id")
    )
)]
async fn get_guest_order(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: Option<AuthenticatedUser>,
    headers: HeaderMap,
) -> Result<Json<GuestOrder>> {
    let mut tx = s.ps.begin_tx(false).await?;
    if !s.erp.guest_order.is_exists(id, tx.as_mut()).await? {
        return AppError::custom(
            CustomErrorCode::OrderNotFound,
            "Guest order is not found.",
        )
        .into_err();
    }
    if authenticated.is_none() {
        match headers.get("X-Sub-Authorization") {
            Some(client_token) => {
              if !s.erp.guest_order.is_token_match(id, client_token.to_str().expect("Sub token header value to str failed!"), tx.as_mut()).await? {
                  return AppError::custom(CustomErrorCode::NoPermission, "You no permission to access!").into_err();
               }
            },
            None => {
                return AppError::custom(CustomErrorCode::NoPermission, "You don't have token to access!").into_err();
            }
          };
    }
    Ok(Json(s.erp.guest_order.get(id, tx.as_mut()).await?.unwrap()))
}

/// get orders
#[utoipa::path(
    get,
    path = "/orders",
    responses(
        (status = 200, description = "get orders history successfully", body = ListSlice<Order>)
    ),
    params(
        Pagination,
        GetOrdersQuery,
    )
)]
async fn get_orders(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(q): Query<GetOrdersQuery>,
    authenticated: AuthenticatedUser,
) -> Result<Json<ListSlice<Order>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let action = authenticated.user.as_action_type(false);
    let items = s.erp.order.get_multiple(&pagination.correct(), &q, action, tx.as_mut()).await?;
    let count = s.erp.order.get_count(&q, action, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { count, items }))
}

/// get guest orders
#[utoipa::path(
    get,
    path = "/guest_orders",
    responses(
        (status = 200, description = "get guest orders successfully", body = ListSlice<Order>)
    ),
    params(
        Pagination,
        GetOrdersQuery,
    )
)]
async fn get_guest_orders(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(q): Query<GetGuestOrdersQuery>,
    authenticated: AuthenticatedUser,
) -> Result<Json<ListSlice<GuestOrder>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let action = authenticated.user.as_action_type(false);
    let items = s.erp.guest_order.get_multiple(&pagination.correct(), &q, action, tx.as_mut()).await?;
    let count = s.erp.guest_order.get_count(&q, action, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { count, items }))
}

/// update order
#[utoipa::path(
    put,
    path = "/orders/{id}",
    responses(
        (status = 200, description = "update order successfully", body = Order)
    ),
    params(
        ("id"=i64, Path, description = "order id")
    )
)]
async fn update_order(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Json(mut body): Json<Order>,
) -> Result<Json<Order>> {
    authenticated.is_update_remove_order()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if !s.erp.order.is_exists(id, tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::OrderNotFound, "Order is not found.").into_err();
    }
    if !s.erp.warehouse.is_linked(WarehouseIsFrom::Order(id), (&authenticated.user).into(), tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::NotLinked, "You not linked to warehouse!").into_err();
    }
    if !s.erp.order.can_access(id, &authenticated.user, tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::NoPermission, "You not the order's owner or admin!").into_err();
    }
    check_order_and_preprocess(s.clone(), &authenticated, false, &mut body, tx.as_mut()).await?;
    let r = s.erp.order.update(id, body, authenticated.user.as_action_type(false), tx.as_mut()).await?.unwrap();
    tx.commit().await?;
    Ok(Json(r))
}

/// inventory list
#[utoipa::path(
    get,
    path = "/inventory",
    responses(
        (status = 200, description = "get inventory list successfully", body = ListSlice<InventoryProduct>)
    ),
    params(
        Pagination,
        GetInventoryQuery,
    )
)]
async fn inventory_list(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(q): Query<GetInventoryQuery>,
    authenticated: AuthenticatedUser,
) -> Result<Json<ListSlice<InventoryProduct>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    let action = authenticated.user.as_action_type(false);
    let items = s.erp.inventory.list(&pagination.correct(), &q, action, tx.as_mut()).await?;
    let count = s.erp.inventory.get_count(&q, action, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { items, count }))
}

/// inventory list in excel file.
#[utoipa::path(
    get,
    path = "/inventory_excel",
    responses(
        (status = 200, description = "get inventory excel successfully", body = Response)
    ),
    params(
        GetInventoryQuery,
    )
)]
async fn inventory_list_excel(
    State(s): State<AppState>,
    Query(q): Query<GetInventoryQuery>,
    authenticated: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    let mut tx = s.ps.begin_tx(false).await?;
    let path = s.erp.inventory.get_excel(&q, authenticated.user.as_action_type(false), tx.as_mut()).await?;
    tx.commit().await?;
    // `File` implements `AsyncRead`
    let file = tokio::fs::File::open(path)
        .await
        .map_err(anyhow::Error::from)?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let headers = [
        (
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"Inventory.xlsx\"",
        ),
    ];
    Ok((headers, body).into_response())
}

async fn check_sku(s: AppState, v: &SKU, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<()> {
    if !s.erp.sku_category.is_exists(v.sku_category_id, &mut *tx).await? {
        return AppError::custom(
            CustomErrorCode::SKUCategoryNotFound,
            "Sku's category id is not found.",
        )
        .into_err();
    }

    if s.erp
        .sku
        .is_exists_name(&v.name, v.sku_category_id, prev, &mut *tx)
        .await?
    {
        return AppError::custom(
            CustomErrorCode::SameObject,
            "Already contains the sku's name.",
        )
        .into_err();
    }

    Ok(())
}

/// add sku
#[utoipa::path(
    post,
    path = "/skus",
    responses(
        (status = 200, description = "add sku successfully", body = SKU)
    ),
)]
async fn add_sku(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(body): Json<SKU>,
) -> Result<Json<SKU>> {
    authenticated.is_manage_sku()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.erp.sku.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::SKULimitExceeded, "SKU count limit exceeded!").into_err();
    }
    check_sku(s.clone(), &body, None, tx.as_mut()).await?;
    let r = s.erp.sku.add(body, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(r))
}

async fn remove_sku_core(s: AppState, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
    if s.erp.sku.is_depend_by_another(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::SomeoneIsDepentIt, "Some one is depent to the sku.").into_err();
    }
    Ok(s.erp.sku.remove(id, notice, &mut *tx).await?)
}

/// remove sku
#[utoipa::path(
    delete,
    path = "/skus/{id}",
    responses(
        (status = 200, description = "remove sku successfully")
    ),
    params(
        ("id"=i64, Path, description = "sku id")
    )
)]
async fn remove_sku(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
) -> Result<StatusCode> {
    authenticated.is_manage_sku()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if remove_sku_core(s.clone(), id, true, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(StatusCode::OK)
    } else {
        AppError::custom(CustomErrorCode::SKUNotFound, "The sku is not found!").into_err()
    }
}

/// clear skus
#[utoipa::path(
    delete,
    path = "/skus",
    responses(
        (status = 200, description = "clear skus successfully")
    ),
    params(
        Pagination,
        GetSKUsQuery,
    )
)]
async fn clear_skus(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Query(q): Query<GetSKUsQuery>,
) -> Result<Json<ClearResult>> {
    authenticated.is_manage_sku()?;
    let mut tx = s.ps.begin_tx(true).await?;
    let mut pagination = Pagination::new(0, 100);
    let mut success = 0;
    let mut failed = 0;
    let mut ids = s.erp.sku.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    while ids.len() > 0 {
        for id in ids {
            match remove_sku_core(s.clone(), id, false, tx.as_mut()).await {
                Ok(_) => success += 1,
                Err(_) => failed += 1,
            }
        }
        pagination.set_offset(failed);
        ids = s.erp.sku.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    }
    tx.commit().await?;
    s.ps.notice(crate::public_system::model::WebSocketFlags::ClearSKUs).await?;
    Ok(Json(ClearResult {
        success,
        failed,
    }))
}

/// get sku
#[utoipa::path(
    get,
    path = "/skus/{id}",
    responses(
        (status = 200, description = "get sku successfully", body = SKU)
    ),
    params(
        ("id"=i64, Path, description = "sku id")
    )
)]
async fn get_sku(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: Option<AuthenticatedUser>,
    headers: HeaderMap,
) -> Result<Json<SKU>> {
    let mut tx = s.ps.begin_tx(false).await?;
    check_token_exists(s.clone(), authenticated.as_ref(), &headers, tx.as_mut()).await?;
    if let Some(v) = s.erp.sku.get(id, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(CustomErrorCode::SKUNotFound, "Sku is not found.").into_err()
    }
}

/// get skus
#[utoipa::path(
    get,
    path = "/skus",
    responses(
        (status = 200, description = "get skus successfully", body = ListSlice<SKU>)
    ),
    params(
        Pagination,
        GetSKUsQuery
    )
)]
async fn get_skus(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(q): Query<GetSKUsQuery>,
    authenticated: Option<AuthenticatedUser>,
    headers: HeaderMap,
) -> Result<Json<ListSlice<SKU>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    check_token_exists(s.clone(), authenticated.as_ref(), &headers, tx.as_mut()).await?;

    let items = s.erp.sku.get_multiple(&pagination.correct(), &q, tx.as_mut()).await?;
    let count = s.erp.sku.get_count(&q, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { items, count }))
}

/// update sku
#[utoipa::path(
    put,
    path = "/skus/{id}",
    responses(
        (status = 200, description = "update sku successfully", body = SKU)
    ),
    params(
        ("id"=i64, Path, description = "sku id")
    )
)]
async fn update_sku(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Json(body): Json<SKU>,
) -> Result<Json<SKU>> {
    authenticated.is_manage_sku()?;
    let mut tx = s.ps.begin_tx(true).await?;
    check_sku(s.clone(), &body, Some(id), tx.as_mut()).await?;
    if let Some(v) = s.erp.sku.update(id, body, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(CustomErrorCode::SKUNotFound, "Sku is not found.").into_err()
    }
}

async fn check_sku_category(s: AppState, v: &SKUCategory, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<()> {
    if s.erp.sku_category.is_exists_name(&v.name, prev, &mut *tx).await? {
        return AppError::custom(
            CustomErrorCode::SameObject,
            "Already contains the sku category's name.",
        )
        .into_err();
    }
    Ok(())
}
/// add sku category
#[utoipa::path(
    post,
    path = "/sku_categories",
    responses(
        (status = 200, description = "add sku category successfully", body = SKUCategory)
    ),
)]
async fn add_sku_category(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(body): Json<SKUCategory>,
) -> Result<Json<SKUCategory>> {
    authenticated.is_manage_sku_category()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if s.erp.sku_category.is_limit_reached(tx.as_mut()).await? {
        return AppError::custom(CustomErrorCode::SKUCategoryLimitExceeded, "SKU category count limit exceeded!").into_err();
    }
    check_sku_category(s.clone(), &body, None, tx.as_mut()).await?;
    let r = s.erp.sku_category.add(body, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(r))
}

async fn remove_sku_category_core(s: AppState, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
    if s.erp.sku_category.is_depend_by_another(id, &mut *tx).await? {
        return AppError::custom(CustomErrorCode::SomeoneIsDepentIt, "Some one is depent to the sku category.").into_err();
    }
    Ok(s.erp.sku_category.remove(id, notice, &mut *tx).await?)
}

/// remove sku category
#[utoipa::path(
    delete,
    path = "/sku_categories/{id}",
    responses(
        (status = 200, description = "remove sku category successfully")
    ),
    params(
        ("id"=i64, Path, description = "sku category id")
    )
)]
async fn remove_sku_category(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    authenticated.is_manage_sku_category()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if remove_sku_category_core(s.clone(), id, true, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(StatusCode::OK)
    } else {
        AppError::custom(CustomErrorCode::SKUCategoryNotFound, "The SKU category is not found.").into_err()
    }
}

/// clear sku categories
#[utoipa::path(
    delete,
    path = "/sku_categories",
    responses(
        (status = 200, description = "clear sku categories successfully")
    ),
    params(
        Pagination,
        GetSKUCategoriesQuery,
    )
)]
async fn clear_sku_categories(
    State(s): State<AppState>,
    authenticated: AuthenticatedUser,
    Query(q): Query<GetSKUCategoriesQuery>,
) -> Result<Json<ClearResult>> {
    authenticated.is_manage_sku_category()?;
    let mut tx = s.ps.begin_tx(true).await?;
    let mut pagination = Pagination::new(0, 100);
    let mut success = 0;
    let mut failed = 0;
    let mut ids = s.erp.sku_category.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    while ids.len() > 0 {
        for id in ids {
            match remove_sku_category_core(s.clone(), id, false, tx.as_mut()).await {
                Ok(_) => success += 1,
                Err(_) => failed += 1,
            }
        }
        pagination.set_offset(failed);
        ids = s.erp.sku_category.get_multiple_ids(&pagination, &q, tx.as_mut()).await?;
    }
    tx.commit().await?;
    s.ps.notice(crate::public_system::model::WebSocketFlags::ClearSKUCategories).await?;
    Ok(Json(ClearResult {
        success,
        failed,
    }))
}

/// get sku category
#[utoipa::path(
    get,
    path = "/sku_categories/{id}",
    responses(
        (status = 200, description = "get sku category successfully", body = SKUCategory)
    ),
    params(
        ("id"=i64, Path, description = "sku category id")
    )
)]
async fn get_sku_category(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: Option<AuthenticatedUser>,
    headers: HeaderMap,
) -> Result<Json<SKUCategory>> {
    let mut tx = s.ps.begin_tx(false).await?;
    check_token_exists(s.clone(), authenticated.as_ref(), &headers, tx.as_mut()).await?;
    if let Some(v) = s.erp.sku_category.get(id, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(
            CustomErrorCode::SKUCategoryNotFound,
            "Sku category is not found.",
        )
        .into_err()
    }
}

/// get sku categories
#[utoipa::path(
    get,
    path = "/sku_categories",
    responses(
        (status = 200, description = "get sku_categories successfully", body = ListSlice<SKUCategory>)
    ),
    params(
        Pagination,
        GetSKUCategoriesQuery
    )
)]
async fn get_sku_categories(
    State(s): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(q): Query<GetSKUCategoriesQuery>,
    authenticated: Option<AuthenticatedUser>,
    headers: HeaderMap,
) -> Result<Json<ListSlice<SKUCategory>>> {
    let mut tx = s.ps.begin_tx(false).await?;
    check_token_exists(s.clone(), authenticated.as_ref(), &headers, tx.as_mut()).await?;
    let items = s.erp.sku_category.get_multiple(&pagination.correct(), &q, tx.as_mut()).await?;
    let count = s.erp.sku_category.get_count(&q, tx.as_mut()).await?;
    tx.commit().await?;
    Ok(Json(ListSlice { items, count }))
}

/// update sku category
#[utoipa::path(
    put,
    path = "/sku_categories/{id}",
    responses(
        (status = 200, description = "update sku category successfully", body = SKUCategory)
    ),
    params(
        ("id"=i64, Path, description = "sku category id")
    )
)]
async fn update_sku_category(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    authenticated: AuthenticatedUser,
    Json(body): Json<SKUCategory>,
) -> Result<Json<SKUCategory>> {
    authenticated.is_manage_sku_category()?;
    let mut tx = s.ps.begin_tx(true).await?;
    if let Some(v) = s.erp.sku_category.update(id, body, tx.as_mut()).await? {
        tx.commit().await?;
        Ok(Json(v))
    } else {
        AppError::custom(
            CustomErrorCode::SKUCategoryNotFound,
            "Sku category is not found.",
        )
        .into_err()
    }
}
