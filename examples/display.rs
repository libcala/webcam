use camera::{Rig, Camera};
use pix::Raster;
use pix::rgb::SRgba8;

// Raster<SRgba8>

enum Event {
    // Rig has discovered a new camera.
    Camera(Camera),
    // 
    Picture((usize, ())),
}

struct State {
}

impl State {
    fn event(&mut self, event: Event) -> bool {
        match event {
            Event::Camera(_) => println!("New camera"),
            Event::Picture(_) => println!("Got picture"),
        }
        true
    }
}

// Asynchronous entry point.
async fn run() {
    let mut state = State {
        
    };
    let mut cams = Vec::<Camera>::new();
    let mut rig = Rig::new();
    while state.event(pasts::wait! {
        Event::Camera((&mut rig).await),
        Event::Picture(pasts::race!(cams)),
    }) {}
}

fn main() {
    pasts::block_on(run());
}

/*    let camera = Camera::new();


	void* output = NULL;
//	uint8_t output[640*480*3]; // RGB
	const char* error;
	int i;

	if((error = car_camera_init(&camera, 0, 640, 480, &output))) {
		printf("Error %s.\n", error);
		return 1;
	}
//	printf("Camera width and height: %dx%d\n", camera.w, camera.h);
	for(i = 0; i < 20; i++) {
		printf("getting a frame....\n");
		if((error = car_camera_loop(&camera))) {
			printf("Error %s.\n", error);
			return 1;
		}
	}
//	for(i = 0; i < 100; i++) printf("%d\n", output[i]);
	if((error = car_camera_kill(&camera))) {
		printf("Error %s.\n", error);
		return 1;
	}
	printf("Success!\n");
	return 0;
}*/
