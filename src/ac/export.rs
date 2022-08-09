use std::io;
extern crate csv;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::thread;

use super::account;

static mut THREADS: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

static MAX_THREAD: i32 = 100;

pub fn run() {
    let data = account::CLIENTS.lock().unwrap().to_vec().clone();
    let len: usize = data.len();
    let max_thread: usize = MAX_THREAD as usize;

    let nodes: usize = len / max_thread;

    let mut counter: usize = 0;

    if len < 1 {
        return;
    }
    // print the headers
    println!("client,available,held,total,locked");

    if len < max_thread {
        export(data);
        return;
    }

    let mut from: usize = 0;
    let mut to;

    loop {
        if counter == max_thread {
            break;
        }
        let new_count = counter + 1;
        to = new_count * nodes;

        if new_count == max_thread {
            to = len;
        }

        let cloned_data = data.clone();

        increase_threads();
        thread::spawn(move || export(cloned_data[from..to].to_vec()));
        counter += 1;

        from = to;
    }

    // wait for threads to finish running
    loop {
        // println!("get_thread_count {}", get_thread_count());
        if counter >= max_thread && get_thread_count() < 1 {
            break;
        }
    }
}

fn export(data: Vec<u16>) {
    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(io::stdout());
    for id in data {
        let acct = account::ACCOUNTS
            .lock()
            .unwrap()
            .read(id, |acc| acc.unwrap().clone());
        csv_writer.serialize(acct).unwrap();
    }
    decrease_threads();
}

fn increase_threads() {
    unsafe {
        let mut t = THREADS.lock().unwrap();
        *t += 1;
    }
}

fn decrease_threads() {
    unsafe {
        let mut t = THREADS.lock().unwrap();
        *t -= 1;
    }
}

fn get_thread_count() -> i32 {
    let t: i32;
    unsafe {
        t = *THREADS.lock().unwrap();
    }

    t
}
