use crate::data_structures::{StreamId, UserId};
use crate::http_server::response::Response;
use crate::services::StreamServiceFactory;
use actix_web::{web, HttpResponse};

pub(crate) async fn play(
    params: web::Path<StreamId>,
    stream_service_factory: web::Data<StreamServiceFactory>,
    user_id: UserId,
) -> Response {
    let stream_id = params.into_inner();
    let stream_service = stream_service_factory
        .create_service(&stream_id, &user_id)
        .await?;

    stream_service.play().await?;

    Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn stop(
    params: web::Path<StreamId>,
    stream_service_factory: web::Data<StreamServiceFactory>,
    user_id: UserId,
) -> Response {
    let stream_id = params.into_inner();
    let stream_service = stream_service_factory
        .create_service(&stream_id, &user_id)
        .await?;

    stream_service.stop().await?;

    Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn play_next(
    params: web::Path<StreamId>,
    stream_service_factory: web::Data<StreamServiceFactory>,
    user_id: UserId,
) -> Response {
    let stream_id = params.into_inner();
    let stream_service = stream_service_factory
        .create_service(&stream_id, &user_id)
        .await?;

    stream_service.play_next().await?;

    Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn play_prev(
    params: web::Path<StreamId>,
    stream_service_factory: web::Data<StreamServiceFactory>,
    user_id: UserId,
) -> Response {
    let stream_id = params.into_inner();
    let stream_service = stream_service_factory
        .create_service(&stream_id, &user_id)
        .await?;

    stream_service.play_prev().await?;

    Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn play_from(
    params: web::Path<StreamId>,
    stream_service_factory: web::Data<StreamServiceFactory>,
    user_id: UserId,
) -> Response {
    let stream_id = params.into_inner();
    let stream_service = stream_service_factory
        .create_service(&stream_id, &user_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}