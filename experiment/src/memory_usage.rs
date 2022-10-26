pub type Kilobytes = libc::c_long;

#[derive(Debug, Copy, Clone)]
pub enum Resource {
    Myself,
    Children,
}

impl Resource {
    fn to_libc(self) -> libc::c_int {
        match self {
            // See the getrusage manpage for the meaning of these values.
            Resource::Myself => 0,
            Resource::Children => -1,
        }
    }
}

pub fn get_maximum_memory_usage(resource: Resource) -> Option<Kilobytes> {
    get_rusage(resource).map(|rusage| rusage.ru_maxrss)
}

fn get_rusage(resource: Resource) -> Option<libc::rusage> {
    // No way around unsafe: we are calling the C API after all.
    unsafe {
        let mut rusage = std::mem::zeroed();
        let ret = libc::getrusage(resource.to_libc(), &mut rusage);
        if ret != 0 {
            return None;
        }
        Some(rusage)
    }
}
