// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use hyper::{
    client::HttpConnector, header::HeaderValue, service::service_fn, Body, Client, Server,
};
use hyper_tls::HttpsConnector;
use pyo3::{prelude::*, types::PyList};
use std::{error::Error, fmt::format, thread, time::Duration};
use tokio::runtime::Runtime;
use tower::make::Shared;

fn main() {
    thread::spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Cannot make runtime")
            .block_on(async {
                let addr = "127.0.0.1:6800".parse().unwrap();
                Server::bind(&addr)
                    .serve(Shared::new(service_fn(|req| {
                        let crop = req.uri() == "/composer/files/new";
                        println!("{}", req.uri());
                        let patchPython = req.uri() == "/_nuxt/ac3b840.js";
                        let new_uri = format!("https://quantum-computing.ibm.com{}", req.uri());
                        async move {
                            let mut conn = HttpConnector::new();
                            conn.enforce_http(false);
                            let client = hyper::Client::builder()
                                .build::<_, hyper::Body>(HttpsConnector::new_with_connector(conn));

                            let mut resp = client
                                .request(
                                    hyper::Request::builder()
                                        .header("X-Origin", "quantum-computing.ibm.com")
                                        .uri(new_uri)
                                        .body(hyper::Body::empty())
                                        .unwrap(),
                                )
                                .await;
                            if let Ok(ref mut resp) = resp {
                                resp.headers_mut().remove("X-Frame-Options");
                                resp.headers_mut().append(
                                    "Access-Control-Allow-Origin",
                                    HeaderValue::from_str("*").unwrap(),
                                );
                                if patchPython {
                                    println!("patching python");
                                    let body = hyper::body::to_bytes(resp.body_mut())
                                        .await
                                        .unwrap()
                                        .to_vec();

                                    let html_str = std::str::from_utf8(&body).unwrap().to_string();
                                    println!("{}", html_str.find("_(\"copy python code\")").is_some());
                                    let fitted = html_str.replacen("C.value.code&&(Object(Be.a)(C.value.code),j.show({message:\"Code copied to clipboard\"}),_(\"copy python code\"))", "window.top.postMessage(C.value.code, '*')", 1);
                                    *resp.body_mut() = Body::from(fitted);
                                }
                                if crop {
                                    let body = hyper::body::to_bytes(resp.body_mut())
                                        .await
                                        .unwrap()
                                        .to_vec();

                                    let html_str = std::str::from_utf8(&body).unwrap().to_string();

                                    let mut parsed =
                                        tl::parse(&html_str, tl::ParserOptions::default()).unwrap();

                                    let head =
                                        parsed.query_selector("head").unwrap().next().unwrap();
                                    let (start, _) = head
                                        .get(parsed.parser())
                                        .unwrap()
                                        .as_tag()
                                        .unwrap()
                                        .boundaries(parsed.parser());

                                    let idx = start as usize + 7;

                                    let fitted =
                                        (&[&html_str[..idx], r#"<script> 
                                        window.onmessage = (msg) => {
                                            if (msg.data === "click") {
                                                let btn = document.querySelectorAll('div.duo--Dropdown--dropdownitems.duo--Dropdown--dropdownitems-small > div:last-child')[2];
                                                console.log(btn);
                                                btn.click();
                                                setTimeout(() => {
                                                //     let str = "";
                                                //     console.log(document.querySelectorAll('div.view-lines.monaco-mouse-cursor-text').length);
                                                //     let text = document.querySelector('div.view-lines.monaco-mouse-cursor-text');
                                                //     text.style.position = 'fixed';
                                                //     text.style.inset = '0';
                                                //     console.log(text);
                                                //     console.log(text.children);
                                                //     for (const div of text.children) {
                                                //         console.log(div);
                                                //         for (const sp of div.children) {
                                                //             console.log(sp);
                                                //             for (const span of sp.children) {
                                                //                 console.log(span);
                                                //                 str += span.innerText;
                                                //             } 
                                                //             str += '\n';
                                                //         } 
                                                //     }
                                                //     window.top.postMessage(str, "*");
                                                    let btn = document.querySelectorAll('div.duo--OverflowMenu__container.duo--OverflowMenu__container--left > button:first-child');
                                                    btn.forEach(b => b.click());
                                                }, 500);
                                            } 
                                        }; </script><style> 
                                          header.cds--header.width-before-scroll-bar.global-header { opacity: 0; pointer-events: none; !important; } 
                                          div.app-menu { opacity: 0; pointer-events: none; !important; } 
                                          div.box-border.grow.shrink.h-full.h-full.overflow-auto { position: fixed; inset: 0; z-index: 99999 } 
                                          div.box-border.grow.shrink.h-full.h-full.overflow-auto ~ * { opacity: 0; pointer-events: none;  }
                                        </style>"#, &html_str[idx..]])
                                            .concat();

                                    *resp.body_mut() = Body::from(fitted);
                                }
                            } else if let Err(ref err) = resp {
                                println!("resp err: {}", err);
                            }

                            resp
                        }
                    })))
                    .await
                    .unwrap();
            });
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![invoke_py])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn invoke_py(code: &str) -> Result<Vec<i64>, String> {
    pyo3::prepare_freethreaded_python();

    let code: String = code.lines().map(|line| format!("    {}\n", line)).collect();

    Python::with_gil(|py| {
        let code = format!(
            r#"def run():
{code}

    from qiskit import Aer, transpile

    circuit.measure_all()
    simulator = Aer.get_backend('aer_simulator')
    circuit = transpile(circuit, simulator)

    result = simulator.run(circuit, shots=10000).result()
    counts = result.get_counts(circuit)
    return list(counts.values())
"#
        );

        println!("{}", code);

        let quant =
            PyModule::from_code(py, &code, "quant.py", "quant").map_err(|err| err.to_string())?;
        let res: Vec<i64> = quant
            .getattr("run")
            .map_err(|err| err.to_string())?
            .call0()
            .map_err(|err| err.to_string())?
            .extract()
            .map_err(|err| err.to_string())?;
        Ok(res)
    })
}
