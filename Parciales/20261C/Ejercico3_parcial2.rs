const N: usize = 10;

fn main() {
    let buffer = Arc::new(Mutex::new(Vec::<u32>::with_capacity(N)));
    let buffer_local = buffer.clone();
    let handle = thread::spawn(move || {
        loop {
            let mut buf = buffer_local.lock().unwrap();
            if buf.len() < N {
                buf.push(rand::thread_rng().gen());
            } else {
                thread::sleep(Duration::from_secs(1));
                drop(buf);
            }
        }
    });

    loop {
        let mut buf = buffer.lock().unwrap();
        if !buf.is_empty() {
            println!("{}", buf.pop().unwrap());
        } else {
            thread::sleep(Duration::from_secs(1));
            drop(buf);
        }
    }

    handle.join().unwrap();
}