use worker::*;

mod routes;
mod crypto;
mod storage;

use routes::{wellknown, activitypub, atproto};

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router
        // Health check
        .get("/health", |_, _| {
            Response::ok("OK")
        })

        // Well-known endpoints
        .get("/.well-known/webfinger", wellknown::webfinger)
        .get("/.well-known/nodeinfo", wellknown::nodeinfo_redirect)
        .get("/nodeinfo/2.1", wellknown::nodeinfo)
        .get("/.well-known/did.json", wellknown::did_document)

        // ActivityPub endpoints
        .get("/users/:username", activitypub::actor)
        .get("/users/:username/inbox", activitypub::inbox_get)
        .post("/users/:username/inbox", activitypub::inbox_post)
        .get("/users/:username/outbox", activitypub::outbox)
        .get("/users/:username/followers", activitypub::followers)
        .get("/users/:username/following", activitypub::following)

        // AT Protocol endpoints
        .get("/xrpc/com.atproto.server.describeServer", atproto::describe_server)
        .post("/xrpc/com.atproto.server.createSession", atproto::create_session)
        .post("/xrpc/com.atproto.server.refreshSession", atproto::refresh_session)
        .post("/xrpc/com.atproto.repo.createRecord", atproto::create_record)
        .get("/xrpc/com.atproto.repo.getRecord", atproto::get_record)
        .get("/xrpc/com.atproto.identity.resolveHandle", atproto::resolve_handle)

        // Run the router
        .run(req, env)
        .await
}
