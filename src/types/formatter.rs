const UNITS: [&str; 5] = ["", "K", "M", "G", "T"];

pub fn human_format(val: f32) -> (f32, &'static str) {
    let mut val = val;

    let mut i = 0;
    for _ in &UNITS {
        if val >= 1024.0 {
            val /= 1024.0;
            i += 1;
        } else {
            break;
        }
    }

    (val, UNITS[i])
}

