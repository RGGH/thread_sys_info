use std::sync::mpsc::channel;
use std::thread;
use sysinfo::System;
use sysinfo::ComponentExt;
use sysinfo::SystemExt;

enum Request {
    LoadAvg,
    Temp,
}

fn main() {
    let (req_tx, req_rx) = channel();
    let (resp_tx, resp_rx) = channel();

    let worker = thread::spawn(move || {
        let mut sys = System::new_all();
        for req in req_rx.iter() {
            sys.refresh_all();

            let msg = match req {
                Request::LoadAvg => {
                    let load_avg = sys.load_average();
                    format!(
                        "{{\"one-minute\":{},\"five-minutes\":{}, \"fifteen-minutes\":{}}}",
                        load_avg.one, load_avg.five, load_avg.fifteen
                    )
                }
                Request::Temp => {
                    let mut coretemps = Vec::new();
                    for component in sys.components() {
                        if component.label().starts_with("coretemp Core") {
                            let temp = component.temperature();
                            coretemps.push(temp);
                        }
                    }
                    // temp_info
                    format!("{coretemps:?}")
                }
            };
            resp_tx.send(msg);
        }
    });

    let printer = thread::spawn(move || {
        for msg in resp_rx.iter() {
            println!("{msg}");
        }
    });

    loop {
        req_tx.send(Request::LoadAvg);
        req_tx.send(Request::Temp);

        thread::sleep(std::time::Duration::from_secs(3));
    }
}
