use smelling_salts::{Device, Watcher};

use std::{
    convert::TryInto,
    future::Future,
    mem,
    os::raw::{c_void, c_int, c_uint},
    pin::Pin,
    task::{Context, Poll},
};

/// Type of the buffer
#[repr(C)]
enum V4l2BufType {
    /// Buffer of a single-planar video capture stream, see Video Capture Interface.
    VIDEO_CAPTURE =	1,
    /// Buffer of a multi-planar video capture stream, see Video Capture Interface.
    VIDEO_CAPTURE_MPLANE = 9,
    /// Buffer of a single-planar video output stream, see Video Output Interface.
    VIDEO_OUTPUT = 2,
    /// Buffer of a multi-planar video output stream, see Video Output Interface.
    VIDEO_OUTPUT_MPLANE =	10,
    /// Buffer for video overlay, see Video Overlay Interface.
    VIDEO_OVERLAY =	3, 	
    /// Buffer of a raw VBI capture stream, see Raw VBI Data Interface.
    VBI_CAPTURE = 	4,
    /// Buffer of a raw VBI output stream, see Raw VBI Data Interface.
    VBI_OUTPUT =	5,
    /// Buffer of a sliced VBI capture stream, see Sliced VBI Data Interface.
    SLICED_VBI_CAPTURE =	6,
    /// Buffer of a sliced VBI output stream, see Sliced VBI Data Interface.
    SLICED_VBI_OUTPUT =	7,
    /// Buffer for video output overlay (OSD), see Video Output Overlay Interface.
    VIDEO_OUTPUT_OVERLAY =	8,
    /// Buffer for Software Defined Radio (SDR) capture stream, see Software Defined Radio Interface (SDR).
    SDR_CAPTURE =	11,
    /// Buffer for Software Defined Radio (SDR) output stream, see Software Defined Radio Interface (SDR).
    SDR_OUTPUT =	12,
}

#[repr(C)]
struct V4l2Capability {
    driver: [u8; 16],    /* i.e. "bttv" */
    card: [u8; 32],      /* i.e. "Hauppauge WinTV" */
    bus_info: [u8; 32],  /* "PCI:" + pci_name(pci_dev) */
    version: u32,        /* should use KERNEL_VERSION() */
    capabilities: u32,   /* Device capabilities */
    reserved: [u32; 4],
}

#[repr(C)]
enum V4l2Field {
    /// Driver can choose from none, top, bottom, interlaced depending on
    /// whatever it thinks is approximate ...
    Any = 0,
    /// This device has no fields
    None = 1,
    /// Top field only
    Top = 2,
    /// Bottom field only
    Bottom = 3,
    /// Both fields interlaced
    Interlaced = 4,
    /// Both fields sequential into one buffer, top-bottom order
    SeqTopBottom = 5,
    /// Same as above + bottom-top order
    SeqBottomTop = 6,
    /// Both fields alternating into separate buffers
    Alternate = 7,
}

#[repr(C)]
enum V4l2Colorspace {
    /// ITU-R 601 -- broadcast NTSC/PAL
    Smpte170M = 1,
    /// 1125-Line (US) HDTV
    Smpte240M = 2,
    /// HD and modern captures.
    Rec709 = 3,
    /// broken BT878 extents (601, luma range 16-253 instead of 16-235)
    Bt878 = 4,
    /// These should be useful.  Assume 601 extents.
    System470M  = 5,
    System470BG = 6,
    /// I know there will be cameras that send this.  So, this is
    /// unspecified chromaticities and full 0-255 on each of the
    /// Y'CbCr components
    Jpeg = 7,
    /// For RGB colourspaces, this is probably a good start.
    Srgb = 8,
}

#[repr(C)]
struct V4l2PixFormat {
    width: u32,
    height: u32,
    pixelformat: u32,
    field: V4l2Field,
    bytesperline: u32, /* for padding, zero if unused */
    sizeimage: u32,
    colorspace: V4l2Colorspace,
    private: u32,       /* private data, depends on pixelformat */
}

#[repr(C)]
struct V4l2Rect {
     left: i32,
     top: i32,
     width: i32,
     height: i32,
}

#[repr(C)]
struct V4l2Clip {
    c: V4l2Rect,
    next: *mut V4l2Clip,
}

#[repr(C)]
struct V4l2Window {
     w: V4l2Rect,
     field: V4l2Field,
     chromakey: u32,
     clips: *mut V4l2Clip,
     clipcount: u32,
     bitmap: *mut c_void,
}

#[repr(C)]
struct V4l2VbiFormat {
    sampling_rate: u32,     /* in 1 Hz */
    offset: u32,
    samples_per_line: u32,
    sample_format: u32,     /* V4L2_PIX_FMT_* */
    start: [i32; 2],
    count: [u32; 2],
    flags: u32,             /* V4L2_VBI_* */
    reserved: [u32; 2],     /* must be zero */
}

#[repr(C)]
union V4l2FormatUnion {
    pix: V4l2PixFormat,     // V4L2_BUF_TYPE_VIDEO_CAPTURE
    win: V4l2Window,        // V4L2_BUF_TYPE_VIDEO_OVERLAY
    vbi: V4l2VbiFormat,     // V4L2_BUF_TYPE_VBI_CAPTURE
    raw_data: [u8; 200],    // user-defined
}

/// Stream data format
#[repr(C)]
struct V4l2Format {
    type_: V4l2BufType,
    fmt: V4l2FormatUnion,
}


/// IOCTL
const fn iow_v(size: usize, num: u8) -> c_int {
    (0x80 << 24) | ((size as c_int & 0x1fff) << 16) | ((b'V' as c_int) << 8) | num
}
const fn ior_v(size: usize, num: u8) -> c_int {
    (0x40 << 24) | ((size as c_int & 0x1fff) << 16) | ((b'V' as c_int) << 8) | num
}
const fn iowr_v(size: usize, num: u8) -> c_int {
    (0xc0 << 24) | ((size as c_int & 0x1fff) << 16) | ((b'V' as c_int) << 8) | num
}
const VIDIOC_STREAMON: c_int = iow_v(mem::size_of::<c_int>(), 18);
const VIDIOC_STREAMOFF: c_int = iow_v(mem::size_of::<c_int>(), 19);
const VIDIOC_QUERYCAP: c_int = ior_v(mem::size_of::<V4l2Capability>(), 0);
const VIDIOC_S_FMT: c_int = iowr_v(mem::size_of::<V4l2Format>(), 5);
const VIDIOC_REQBUFS: c_int = iowr_v(mem::size_of::<v4l2_requestbuffers>(), 8);
const VIDIOC_QUERYBUF: c_int = iowr_v(mem::size_of::<v4l2_buffer>(), 9);
const VIDIOC_QBUF: c_int = iowr_v(mem::size_of::<v4l2_buffer>(), 15);
const VIDIOC_DQBUF: c_int = iowr_v(mem::size_of::<v4l2_buffer>(), 17);

fn xioctl(fd: c_int, request: c_int, arg: *mut c_void) -> c_int {
    loop {
        match ioctl(fd, request, arg) {
            -1 if EINTR == errno => {}
            r => break r,
        }
    }
}

#[inline(always)]
fn errno() -> c_int {
    unsafe { *__errno_location() }
}

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    fn munmap(addr: *mut c_void, length: usize) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn __errno_location() -> *mut c_int;
}

/// All cameras / webcams that are connected to the operating system.
pub struct Rig {
}

impl Rig {
    pub fn new() -> Self {
        Rig {
        }
    }
}

/// A camera / webcam in the `Rig`.
pub struct Camera {
	// Linux specific
	buffer: *mut c_void,
	buf: v4l2_buffer,

	// 
	data: *mut c_void, // JPEG file data
	size: u32, // Size of JPEG file
}

impl Camera {
    pub fn new(w: u16, h: u16, output: *mut *mut c_void) -> Option<Camera>
    {
	    // Open the device
        let filename = "/dev/video0";
        let fd = match OpenOptions::new()
            .read(true)
            .append(true)
            .mode(0)
            .custom_flags(0x0004 /*O_NONBLOCK*/)
            .open(filename)
        {
            Ok(f) => f.into_raw_fd(),
            Err(e) => Error::Io(e),
        };
        if fd == -1 {
            return None;
        }
        // FIXME: Do I need to set asynchronous on FD?

	    

	    // Is it available?
	    let caps: V4l2Capability = MaybeUninit::uninit();
	    if (xioctl(fd, VIDIOC_QUERYCAP, caps.as_mut_ptr()) == -1) {
		    ERROR("Querying Capabilites\n");
		    return car_error;
	    }

	    // Set image format.
	    let fmt = V4l2Format {
	        type_: V4l2BufType::VIDEO_CAPTURE,
	        fmt: V4l2FormatUnion {
	            pix: V4l2PixFormat {
            	    width: w,
	                height: h,
	                pixelformat: V4L2_PIX_FMT_MJPEG,
	                field: V4L2_FIELD_NONE,
	            },
	        },
	    };

	    if (-1 == xioctl(fd, VIDIOC_S_FMT, &fmt)) {
		    ERROR("Error setting Pixel Format\n");
		    return car_error;
	    }

	    // Request a video capture buffer.
	    struct v4l2_requestbuffers req;
	    CLEAR(req);
	    req.count = 1;
	    req.type = V4l2BufType::VIDEO_CAPTURE;
	    req.memory = V4L2_MEMORY_MMAP;
	     
	    if (-1 == xioctl(fd, VIDIOC_REQBUFS, &req))
	    {
		    ERROR("Requesting Buffer\n");
		    return car_error;
	    }

	    // Query buffer
	    CLEAR(buf);
	    buf.type = V4l2BufType::VIDEO_CAPTURE;
	    buf.memory = V4L2_MEMORY_MMAP;
	    buf.index = 0;
	    if(-1 == xioctl(fd, VIDIOC_QUERYBUF, &buf)) {
		    ERROR("Querying Buffer\n");
		    return car_error;
	    }
	    *output = mmap (NULL, buf.length, PROT_READ | PROT_WRITE, MAP_SHARED,
		    fd, buf.m.offset);
	    camera->size = buf.length;

	    // Start the capture:
	    CLEAR(buf);
	    buf.type = V4l2BufType::VIDEO_CAPTURE;
	    buf.memory = V4L2_MEMORY_MMAP;
	    buf.index = 0;

	    if (xioctl(fd, VIDIOC_QBUF, &buf) == -1) {
		    ERROR("VIDIOC_QBUF");
		    return car_error;
	    }

	    enum v4l2_buf_type type;
	    type = V4l2BufType::VIDEO_CAPTURE;
	    if (xioctl(fd, VIDIOC_STREAMON, &type) == -1) {
		    ERROR("VIDIOC_STREAMON");
		    return car_error;
	    }
	    return NULL;
    }
}

impl Future for Camera {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &Context<'_>) -> Poll<Self::Output> {
	    if(r == -1) {
		    if (errno() == EINTR) goto CAR_CAMERA_LOOP;
		    ERROR("Waiting for Frame\n");
		    return car_error;
	    }

	    CLEAR(buf);
	    buf.type = V4l2BufType::VIDEO_CAPTURE;
	    buf.memory = V4L2_MEMORY_MMAP;
	    if(xioctl(fd, VIDIOC_DQBUF, &buf) == -1) {
		    if(errno == EAGAIN) goto CAR_CAMERA_LOOP;
		    ERROR("Retrieving Frame %s\n", strerror(errno));
		    close(fd);
		    return car_error;
	    }

	    if (xioctl(fd, VIDIOC_QBUF, &buf) == -1) {
		    ERROR("VIDIOC_QBUF");
		    return car_error;
	    }
	    return NULL;
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
	    let type_ = V4l2BufType::VIDEO_CAPTURE;
	    if (xioctl(fd, VIDIOC_STREAMOFF, &type_) == -1) {
		    ERROR("VIDIOC_STREAMOFF");
		    return car_error;
	    }
	    if (munmap(self.buffer, self.size) == -1) {
		    ERROR("munmap");
		    return car_error;
	    }
	    if (close(fd) == -1) {
		    ERROR("close");
		    return car_error;
	    }
	    return NULL;
    }
}
