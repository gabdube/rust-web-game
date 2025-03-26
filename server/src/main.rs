mod utils;
mod messages;
mod assets;
mod assets_reload;

//
//
//

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;

use hyper::body::Bytes;
use hyper::upgrade::Upgraded;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_util::rt::TokioIo;

use hyper_tungstenite::{tungstenite, HyperWebsocket, WebSocketStream};
use tungstenite::Message;

use tokio::net::TcpListener;

use futures::{stream::StreamExt, SinkExt};
use http_body_util::Full;

/// Build path for the application
pub const BUILD_PATH: &'static str = "./build/";

/// Alias for a generic error type
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Alias for the web socket stream
type WebSocket = WebSocketStream<TokioIo<Upgraded>>;

struct BasicServer {
    assets: assets::AssetCache
}

impl BasicServer {

    pub fn new() -> Self {
        let assets = assets::AssetCache::new();
        assets_reload::reload_assets(&assets);
        BasicServer {
            assets,
        }
    }
    
    //
    // HTTP
    //

    fn serve_404(&self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("404")))
            .unwrap()
    }

    fn serve_assets(&self, req: Request<IncomingBody>) -> Option<Response<Full<Bytes>>> {
        let path = req.uri().path();
        let local_path = utils::web_path_to_local_path(path);
        let asset = self.assets.fetch_asset(local_path)?;
        let asset_bytes = Full::new(Bytes::from(asset.bytes()));

        let response = Response::builder()
            .header("Content-Type", asset.mime())
            .body(asset_bytes)
            .unwrap();

        Some(response)
    }


    //
    // Websocket
    //

    async fn websocket_client_message(&self, message: Message) -> Result<(), Error> {
        match message {
            Message::Text(_msg) => {
                eprintln!("Received unknown text message");
            },
            Message::Binary(_msg) => {
                eprintln!("Received unknown binary message");
            },
            Message::Close(_msg) => {},
            Message::Ping(_) => {},
            Message::Pong(_) => {},
            Message::Frame(_) => { unreachable!(); }
        }

        Ok(())
    }

    async fn websocket_server_messages(&self, websocket: &mut WebSocket) -> Result<(), Error> {
        let files = self.assets.updated_files();
        if files.len() > 0 {
            for file in files {
                websocket.send(messages::file_changed(file)).await?;
            }
        }

        Ok(())
    }

    async fn serve_websocket(self, websocket: HyperWebsocket) -> Result<(), Error> {
        let mut websocket = websocket.await?;

        loop {
            let out = tokio::time::timeout(Duration::from_millis(100), websocket.next()).await;
            if let Ok(Some(message)) = out {
                match message {
                    Err(_) | Ok(Message::Close(_)) => { break; },
                    Ok(msg) => { self.websocket_client_message(msg).await?; },
                }
            }

            self.websocket_server_messages(&mut websocket).await?;
        }

        Ok(())
    }

    fn handle_websocket(&self, mut req: Request<IncomingBody>) -> Result<Response<Full<Bytes>>, Error> {        
        let (response, websocket) = hyper_tungstenite::upgrade(&mut req, None)?;
        
        // Spawn a task to handle the websocket connection.
        let service = self.clone();
        tokio::spawn(async move {
            if let Err(e) = service.serve_websocket(websocket).await {
                eprintln!("Error in websocket connection: {e}");
            }
        });

        Ok(response)
    }


}

impl Service<Request<IncomingBody>> for BasicServer {
    type Response = Response<Full<Bytes>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let response = match hyper_tungstenite::is_upgrade_request(&req) {
            true => self.handle_websocket(req),
            false => Ok(self.serve_assets(req).unwrap_or_else(|| self.serve_404() )),
        };

        Box::pin(async { response }) 
    }
}


impl Clone for BasicServer {
    fn clone(&self) -> Self {
        BasicServer {
            assets: self.assets.clone()
        }
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    let service = BasicServer::new();

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let service = service.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
