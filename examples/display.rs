use camera::Camera;

fn main() {
    let camera = Camera::new();

	car_camera_t camera;
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
}
