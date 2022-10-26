pub type Kilobytes = libc::c_long;

#[derive(Debug, Copy, Clone)]
pub enum Resource {
    Myself,
    Children
}

impl Resource {
    fn to_libc(self) -> libc::c_int {
        match self {
            Resource::Myself => { 0}
            Resource::Children => { -1}
        }
    }
}

pub fn get_maximum_memory_usage_all_resources() -> Option<Kilobytes> {
    let myself = get_maximum_memory_usage(Resource::Myself);
    let children = get_maximum_memory_usage(Resource::Children);
    myself.zip(children).map(|(a_kb, b_kb)| std::cmp::max(a_kb, b_kb))
}

pub fn get_maximum_memory_usage(resource: Resource) -> Option<Kilobytes> {
    get_rusage(resource).map(|rusage| rusage.ru_maxrss)
}

fn get_rusage(resource: Resource) -> Option<libc::rusage> {
    unsafe {
        let mut rusage = std::mem::zeroed();
        // -1 is RUSAGE_CHILDREN, which means to get the rusage for all children
        // (and grandchildren, etc) processes that have respectively terminated
        // and been waited for.
        let retval = libc::getrusage(resource.to_libc(), &mut rusage);
        if retval != 0 {
            return None;
        }
        Some(rusage)
    }
}