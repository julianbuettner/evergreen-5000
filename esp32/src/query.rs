use embedded_svc::http::{self, client::*, status, Headers, Status};
use embedded_svc::io::Read;
use embedded_svc::utils::io;
use esp_idf_svc::http::client::*;

pub fn content() -> String {
    let url = String::from("https://whatthecommit.com/index.txt");

    println!("About to fetch content from {}", url);
    let mut client = Client::wrap(EspHttpConnection::new(&Configuration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    }).unwrap());

    let mut response = client.get(&url).unwrap().submit().unwrap();
    let mut body = [0_u8; 3048];
    let read = io::try_read_full(&mut response, &mut body).map_err(|err| err.0).unwrap();

    let message = String::from_utf8_lossy(&body[..read]).into_owned();
    println!("Body (truncated to 3K):\n{:?}", message);

    // Complete the response
    while response.read(&mut body).unwrap() > 0 {}

    message
}
