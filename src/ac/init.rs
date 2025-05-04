use std::error::Error;

pub fn run(fish: bool, dev: bool) -> Result<(), Box<dyn Error>> {
    let init_str;

    if fish {
        if dev {
            init_str = include_str!("../init/init_dev.fish");
        } else {
            init_str = include_str!("../init/init.fish");
        }
    }
    // bash
    else {
        if dev {
            init_str = include_str!("../init/init_dev.bash");
        } else {
            init_str = include_str!("../init/init.bash");
        }
    }
    println!("{}", init_str);

    return Ok(());
}
