//! Sample code for  
//! <https://stackoverflow.com/questions/79084097/weird-behaviour-in-emmbedding-sveltekit-into-rust-may-minihttp>
//!

use std::io;

use log::{debug, info};
use may_minihttp::{self as mayhttp, HttpServiceFactory as _};
use mime_mapper::MimeMapper;
use rust_embed::RustEmbed;

mod mime_mapper;

/// true: redirect directory paths to /
/// false: use /index.html content for no index.html in sub-directories (a/ or a/index.html)
const REDIRECT_INDEX: bool = true;

const SERVER_ADDRESS: &str = "0.0.0.0:8800";

#[derive(RustEmbed)]
#[folder = "src/svelte/build/"]
struct Assets;

struct Router;

impl mayhttp::HttpService for Router {
    fn call(
        &mut self,
        req: mayhttp::Request<'_, '_, '_>,
        rsp: &mut mayhttp::Response<'_>,
    ) -> io::Result<()> {
        let mut redirect = false;
        let mut path;
        let asset = match req.path() {
            "/" | "/index.html" => {
                path = "index.html".to_string();
                Assets::get("index.html")
            }
            _ => {
                path = req.path().trim_start_matches('/').to_string();
                if path.ends_with('/') {
                    path += "index.html";
                    redirect = true;
                    debug!("{path} extended for index.html");
                } else if !path.contains('.') {
                    path += "/index.html";
                    redirect = true;
                    debug!("{path} extended for /index.html");
                }
                let mut asset = Assets::get(&path);
                if asset.is_some() && REDIRECT_INDEX && redirect {
                    redirect = false;
                    debug!("{path} asset found");
                } else if asset.is_none() && !REDIRECT_INDEX && redirect {
                    asset = Assets::get("index.html");
                    debug!("{path} retrieves /index.html asset");
                }
                asset
            }
        };

        if let Some(content) = asset {
            let content_type = MimeMapper::instance()
                .get_or_insert(mime_guess::from_path(&path).first_or_octet_stream());
            // or:
            // let content_type =
            //     mime_mapper::mime_map(&mime_guess::from_path(&path).first_or_octet_stream());
            rsp.header(content_type)
                .body_mut()
                .extend_from_slice(&content.data);
            info!("{path} 200 OK");
        } else if redirect {
            rsp.header("Location: /")
                .status_code(307, "Temporary Redirect")
                .body("Temporary Redirect");
        } else {
            // Not Found
            rsp.status_code(404, "Not Found").body("Not Found");
            info!("{path} 404 Not Found");
        }

        Ok(())
    }
}

struct HttpServer;

impl mayhttp::HttpServiceFactory for HttpServer {
    type Service = Router;

    fn new_service(&self, _: usize) -> Self::Service {
        Router
    }
}

fn main() {
    let _ = may::config().set_pool_capacity(500).set_stack_size(0x1000);

    env_logger::init();

    info!("starting server");

    let http_server = HttpServer;
    let server = http_server.start(SERVER_ADDRESS).unwrap();

    info!("server listening at: {SERVER_ADDRESS}");

    server.join().unwrap();
}
