use std::io::File;
use std::os;
use std::io::Command;

fn main(){
	//user-customisable variables:
	let max_brightness = 937;
	let steps : &[int] = [0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, max_brightness];
	let brightness_path = "/sys/class/backlight/intel_backlight/brightness";
	let display = "eDP1";

	let path = Path::new(brightness_path);
	let args = os::args();
	let verbose = args.contains(&"-v".to_string());
	let quiet = args.contains(&"-q".to_string());
	let linear = args.contains(&"-l".to_string());

	let mut file = match File::open(&path) {
		Err(why) => fail!("couldn't open {}: {}", brightness_path, why.desc),
        Ok(file) => file,
    };
    
    let read_brightness = match file.read_to_string() {
        Err(why) => fail!("couldn't read {}: {}", brightness_path, why.desc),
        Ok(read_brightness) => {
        	if verbose {
        		println!("brightness reads \"{}\"", read_brightness.as_slice().trim());
        	};
           	read_brightness
        },
    };

    let brightness : int = match from_str(read_brightness.as_slice().trim()) {
        None => {
        	println!("{} does not contain a valid integer", brightness_path);
        	-1
        },
        Some(brightness) => brightness,
    };

    let mut new_brightness = -1;
	if args.contains(&"-inc".to_string()) {
		if brightness == max_brightness{
			new_brightness = brightness;
		} else {
			for val in steps.iter() {
				if brightness < *val {
					new_brightness = *val;
					break;
				}
			}
		}
    }
    else if args.contains(&"-dec".to_string()) {
    	if brightness == 0 {
    		new_brightness = 0;
    	} else {
	    	for val in steps.iter().rev() {
	    		if brightness > *val {
	    			new_brightness = *val;
	    			break;
	    		}
	    	}
	    }
    }

	if new_brightness >= 0 {
	    match Command::new("/usr/bin/xrandr").args(["--output".as_slice(), display.as_slice(), "--set", "BACKLIGHT", new_brightness.to_string().as_slice() ]).spawn() {	
	        Err(e) => {
	            fail!("failed to execute xrandr: {}", e)
	        },
	        Ok(_) => {
		        if verbose {
		        	println!("setting brightness value to {} in {}", new_brightness, brightness_path);
		        };
		        if ! quiet {
		        	let brightness_percent = if new_brightness < 2 {
		        		new_brightness
		        	} else if linear {
		        		(new_brightness as f32 / max_brightness as f32 * 100f32) as int
		        	} else {
		        		((new_brightness as f32).log2() / (max_brightness as f32).log2() * 100f32) as int
					};

					match Command::new("/usr/bin/notify-send").args([" ".as_slice(), "-i", "display-brightness-symbolic", "-h", format!("int:value:{}", brightness_percent).as_slice(), "-h", "string:synchronous:brightness"]).spawn() {
					    Ok(child) => child,
					    Err(e) => fail!("failed to execute notify-send: {}", e),
					};
		        }
		    }
	    }
	} else {
		if verbose {
			println!("did not set brightness");
		}
		println!("Usage:\n\t-inc\tIncrease brightness\n\t-dec\tDecrease brightness\n\t-v\tVerbose output\n\t-q\tno OSD\n\t-l\tlinear OSD");
	}
}