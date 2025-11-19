use worker::*;

mod routes;
mod crypto;
mod storage;

use routes::{wellknown, activitypub, atproto, oauth};

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router
        // Health check
        .get_async("/health", |_, _| async {
            Response::ok("OK")
        })

        // Well-known endpoints
        .get_async("/.well-known/webfinger", wellknown::webfinger)
        .get_async("/.well-known/nodeinfo", wellknown::nodeinfo_redirect)
        .get_async("/nodeinfo/2.1", wellknown::nodeinfo)
        .get_async("/.well-known/did.json", wellknown::did_document)

        // ActivityPub endpoints
        .get_async("/users/:username", activitypub::actor)
        .get_async("/users/:username/inbox", activitypub::inbox_get)
        .post_async("/users/:username/inbox", activitypub::inbox_post)
        .get_async("/users/:username/outbox", activitypub::outbox)
        .get_async("/users/:username/followers", activitypub::followers)
        .get_async("/users/:username/following", activitypub::following)

        // AT Protocol endpoints
        .get_async("/xrpc/com.atproto.server.describeServer", atproto::describe_server)
        .post_async("/xrpc/com.atproto.server.createSession", atproto::create_session)
        .post_async("/xrpc/com.atproto.server.refreshSession", atproto::refresh_session)
        .post_async("/xrpc/com.atproto.repo.createRecord", atproto::create_record)
        .get_async("/xrpc/com.atproto.repo.getRecord", atproto::get_record)
        .get_async("/xrpc/com.atproto.identity.resolveHandle", atproto::resolve_handle)

        // OAuth endpoints
        .get_async("/.well-known/oauth-authorization-server", oauth::metadata)
        .post_async("/oauth/device", oauth::device_authorize)
        .get_async("/oauth/device/verify", oauth::device_verify_page)
        .post_async("/oauth/device/verify", oauth::device_verify_submit)
        .post_async("/oauth/token", oauth::token)
        .post_async("/oauth/revoke", oauth::revoke)
        .post_async("/oauth/introspect", oauth::introspect)
        .post_async("/oauth/register", oauth::register_client)
        .get_async("/oauth/clients", oauth::list_clients)
        .delete_async("/oauth/clients/:client_id", oauth::delete_client)

        // Run the router
        .run(req, env)
        .await
}
