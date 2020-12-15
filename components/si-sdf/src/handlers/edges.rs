use crate::data::{Connection, Db};
use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models::{edge, get_model, upsert_model, Edge};


pub async fn delete(
    edge_id: String,
    db: Db,
    nats: Connection,
    token: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "edges",
        "delete",
    )
    .await?;

    let mut edge: Edge = get_model(&db, &edge_id, &claim.billing_account_id)
        .await
        .map_err(HandlerError::from)?;
    edge.si_storable.deleted = true;
    upsert_model(&db, &nats, &edge.id, &edge)
        .await
        .map_err(HandlerError::from)?;

    let reply = edge::DeleteReply { edge };

    Ok(warp::reply::json(&reply))
}


pub async fn all_predecessors(
    db: Db,
    token: String,
    request: edge::AllPredecessorsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "edges",
        "allPredecessors",
    )
    .await?;

    let edges = if let Some(object_id) = request.object_id {
        Edge::all_predecessor_edges_by_object_id(&db, request.edge_kind, &object_id)
            .await
            .map_err(HandlerError::from)?
    } else if let Some(node_id) = request.node_id {
        Edge::all_predecessor_edges_by_node_id(&db, request.edge_kind, &node_id)
            .await
            .map_err(HandlerError::from)?
    } else {
        return Err(warp::reject::custom(HandlerError::InvalidRequest));
    };

    let reply = edge::AllPredecessorsReply { edges };

    Ok(warp::reply::json(&reply))
}


pub async fn all_successors(
    db: Db,
    token: String,
    request: edge::AllSuccessorsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "edges",
        "allSuccessors",
    )
    .await?;

    let edges = if let Some(object_id) = request.object_id {
        Edge::all_successor_edges_by_object_id(&db, request.edge_kind, &object_id)
            .await
            .map_err(HandlerError::from)?
    } else if let Some(node_id) = request.node_id {
        Edge::all_successor_edges_by_node_id(&db, request.edge_kind, &node_id)
            .await
            .map_err(HandlerError::from)?
    } else {
        return Err(warp::reject::custom(HandlerError::InvalidRequest));
    };

    let reply = edge::AllSuccessorsReply { edges };

    Ok(warp::reply::json(&reply))
}
