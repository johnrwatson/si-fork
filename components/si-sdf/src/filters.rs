use sodiumoxide::crypto::secretbox;
use warp::{filters::BoxedFilter, Filter};

use si_data::{EventLogFS, NatsConn, PgPool};
use si_model::Veritech;

use crate::handlers;

pub fn api(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
    _event_log_fs: &EventLogFS,
    secret_key: &secretbox::Key,
) -> BoxedFilter<(impl warp::Reply,)> {
    signup_dal(pg, nats_conn, veritech)
        .or(session_dal(pg, secret_key))
        .or(application_dal(pg, nats_conn, veritech))
        .or(application_context_dal(pg, nats_conn, veritech))
        .or(editor_dal(pg, nats_conn, veritech))
        .or(schematic_dal(pg, nats_conn, veritech))
        .or(attribute_dal(pg, nats_conn, veritech))
        .or(secret_dal(pg, nats_conn))
        .or(updates(pg, nats_conn))
        .or(cli(pg, nats_conn, veritech))
        .recover(handlers::handle_rejection)
        .boxed()
}

// The Web Socket CLI API
pub fn cli(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("cli")
        .and(warp::ws())
        .and(with_pg(pg.clone()))
        .and(with_nats_conn(nats_conn.clone()))
        .and(with_veritech(veritech.clone()))
        .and(warp::query::<crate::update::WebsocketToken>())
        .and_then(handlers::cli::cli)
        .boxed()
}

// The Web Socket Update API
pub fn updates(pg: &PgPool, nats_conn: &NatsConn) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("updates")
        .and(warp::ws())
        .and(with_pg(pg.clone()))
        .and(with_nats_conn(nats_conn.clone()))
        .and(warp::query::<crate::update::WebsocketToken>())
        .and_then(handlers::updates::update)
        .boxed()
}

// Session DAL
pub fn session_dal(pg: &PgPool, secret_key: &secretbox::Key) -> BoxedFilter<(impl warp::Reply,)> {
    session_dal_login(pg.clone(), secret_key.clone())
        .or(session_dal_restore_authentication(pg.clone()))
        .or(session_dal_get_defaults(pg.clone()))
        .boxed()
}

pub fn session_dal_login(
    pg: PgPool,
    secret_key: secretbox::Key,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("sessionDal" / "login")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_secret_key(secret_key))
        .and(warp::body::json::<handlers::session_dal::LoginRequest>())
        .and_then(handlers::session_dal::login)
        .boxed()
}

pub fn session_dal_restore_authentication(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("sessionDal" / "restoreAuthentication")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::session_dal::restore_authentication)
        .boxed()
}

pub fn session_dal_get_defaults(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("sessionDal" / "getDefaults")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::session_dal::get_defaults)
        .boxed()
}

// Attribute DAL
pub fn attribute_dal(
    pg: &PgPool,
    _nats_conn: &NatsConn,
    _veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    attribute_dal_get_object_list(pg.clone()).boxed()
}

pub fn attribute_dal_get_object_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("attributeDal" / "getObjectList")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<handlers::attribute_dal::GetObjectListRequest>())
        .and_then(handlers::attribute_dal::get_object_list)
        .boxed()
}

// Schematic DAL
pub fn schematic_dal(
    pg: &PgPool,
    nats_conn: &NatsConn,
    _veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    schematic_dal_get_application_system_schematic(pg.clone())
        .or(schematic_dal_connection_create(
            pg.clone(),
            nats_conn.clone(),
        ))
        .boxed()
}

pub fn schematic_dal_get_application_system_schematic(
    pg: PgPool,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("schematicDal" / "getApplicationSystemSchematic")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<
            handlers::schematic_dal::GetApplicationSystemSchematicRequest,
        >())
        .and_then(handlers::schematic_dal::get_application_system_schematic)
        .boxed()
}

pub fn schematic_dal_connection_create(
    pg: PgPool,
    nats_conn: NatsConn,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("schematicDal" / "connectionCreate")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<
            handlers::schematic_dal::ConnectionCreateRequest,
        >())
        .and_then(handlers::schematic_dal::connection_create)
        .boxed()
}

// Editor DAL
pub fn editor_dal(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    editor_dal_node_create_for_application(pg.clone(), nats_conn.clone(), veritech.clone())
        .or(editor_dal_update_node_position(
            pg.clone(),
            nats_conn.clone(),
        ))
        .boxed()
}

pub fn editor_dal_node_create_for_application(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("editorDal" / "nodeCreateForApplication")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<
            handlers::editor_dal::NodeCreateForApplicationRequest,
        >())
        .and_then(handlers::editor_dal::node_create_for_application)
        .boxed()
}

pub fn editor_dal_update_node_position(
    pg: PgPool,
    nats_conn: NatsConn,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("editorDal" / "updateNodePosition")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<
            handlers::editor_dal::UpdateNodePositionRequest,
        >())
        .and_then(handlers::editor_dal::update_node_position)
        .boxed()
}

// Application Context DAL
pub fn application_context_dal(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    application_context_dal_get_application_context(pg.clone())
        .or(application_context_dal_get_change_set_and_edit_session(
            pg.clone(),
        ))
        .or(
            application_context_dal_create_edit_session_and_get_change_set(
                pg.clone(),
                nats_conn.clone(),
            ),
        )
        .or(application_context_dal_create_change_set_and_edit_session(
            pg.clone(),
            nats_conn.clone(),
        ))
        .or(application_context_dal_create_edit_session(
            pg.clone(),
            nats_conn.clone(),
        ))
        .or(application_context_dal_cancel_edit_session(
            pg.clone(),
            nats_conn.clone(),
            veritech.clone(),
        ))
        .boxed()
}

pub fn application_context_dal_get_application_context(
    pg: PgPool,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("applicationContextDal" / "getApplicationContext")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<
            handlers::application_context_dal::GetApplicationContextRequest,
        >())
        .and_then(handlers::application_context_dal::get_application_context)
        .boxed()
}

pub fn application_context_dal_create_change_set_and_edit_session(
    pg: PgPool,
    nats_conn: NatsConn,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("applicationContextDal" / "createChangeSetAndEditSession")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<
            handlers::application_context_dal::CreateChangeSetAndEditSessionRequest,
        >())
        .and_then(handlers::application_context_dal::create_change_set_and_edit_session)
        .boxed()
}

pub fn application_context_dal_cancel_edit_session(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("applicationContextDal" / "cancelEditSession")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<
            handlers::application_context_dal::CancelEditSessionRequest,
        >())
        .and_then(handlers::application_context_dal::cancel_edit_session)
        .boxed()
}

pub fn application_context_dal_create_edit_session(
    pg: PgPool,
    nats_conn: NatsConn,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("applicationContextDal" / "createEditSession")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<
            handlers::application_context_dal::CreateEditSessionRequest,
        >())
        .and_then(handlers::application_context_dal::create_edit_session)
        .boxed()
}

pub fn application_context_dal_create_edit_session_and_get_change_set(
    pg: PgPool,
    nats_conn: NatsConn,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("applicationContextDal" / "createEditSessionAndGetChangeSet")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<
            handlers::application_context_dal::CreateEditSessionAndGetChangeSetRequest,
        >())
        .and_then(handlers::application_context_dal::create_edit_session_and_get_change_set)
        .boxed()
}

pub fn application_context_dal_get_change_set_and_edit_session(
    pg: PgPool,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("applicationContextDal" / "getChangeSetAndEditSession")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<
            handlers::application_context_dal::GetChangeSetAndEditSessionRequest,
        >())
        .and_then(handlers::application_context_dal::get_change_set_and_edit_session)
        .boxed()
}

// Application DAL
pub fn application_dal(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    application_dal_create_application(pg.clone(), nats_conn.clone(), veritech.clone())
        .or(application_dal_list_applications(pg.clone()))
        .boxed()
}

pub fn application_dal_list_applications(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("applicationDal" / "listApplications")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<
            handlers::application_dal::ListApplicationsRequest,
        >())
        .and_then(handlers::application_dal::list_applications)
        .boxed()
}

pub fn application_dal_create_application(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("applicationDal" / "createApplication")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<
            handlers::application_dal::CreateApplicationRequest,
        >())
        .and_then(handlers::application_dal::create_application)
        .boxed()
}

// Signup DAL
pub fn signup_dal(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    signup_dal_create_billing_account(pg.clone(), nats_conn.clone(), veritech.clone()).boxed()
}

pub fn signup_dal_create_billing_account(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("signupDal" / "createBillingAccount")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::body::json::<handlers::signup_dal::CreateRequest>())
        .and_then(handlers::signup_dal::create_billing_account)
        .boxed()
}

// Secret DAL
pub fn secret_dal(pg: &PgPool, nats_conn: &NatsConn) -> BoxedFilter<(impl warp::Reply,)> {
    secret_dal_get_public_key(pg.clone())
        .or(secret_dal_create_secret(pg.clone(), nats_conn.clone()))
        .or(secret_dal_list_secrets_for_workspace(pg.clone()))
        .boxed()
}

pub fn secret_dal_get_public_key(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("secretDal" / "getPublicKey")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::secret_dal::get_public_key)
        .boxed()
}

pub fn secret_dal_create_secret(
    pg: PgPool,
    nats_conn: NatsConn,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("secretDal" / "createSecret")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<handlers::secret_dal::CreateSecretRequest>())
        .and_then(handlers::secret_dal::create_secret)
        .boxed()
}

pub fn secret_dal_list_secrets_for_workspace(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("secretDal" / "listSecretsForWorkspace")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<
            handlers::secret_dal::ListSecretsForWorkspaceRequest,
        >())
        .and_then(handlers::secret_dal::list_secrets_for_workspace)
        .boxed()
}

fn with_pg(
    pg: PgPool,
) -> impl Filter<Extract = (PgPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pg.clone())
}

fn with_nats_conn(
    nats_conn: NatsConn,
) -> impl Filter<Extract = (NatsConn,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || nats_conn.clone())
}

#[allow(dead_code)]
fn with_event_log_fs(
    event_log_fs: EventLogFS,
) -> impl Filter<Extract = (EventLogFS,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || event_log_fs.clone())
}

fn with_veritech(
    veritech: Veritech,
) -> impl Filter<Extract = (Veritech,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || veritech.clone())
}

fn with_secret_key(
    secret_key: secretbox::Key,
) -> impl Filter<Extract = (secretbox::Key,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || secret_key.clone())
}

#[allow(dead_code)]
fn with_string(
    thingy: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || thingy.clone())
}
