pub struct Params {
    pub turns: u32,
    pub threads: u32,
    pub image_width: u32,
    pub image_height: u32,
}

pub struct Subscription {
    pub worker_ip_address: String,
    pub function: String,
}

pub struct Broker{
    
}

// using static because string literal will live for entire program duration
impl Broker {
    pub const SUBSCRIBE: &'static str = "Broker.Subscribe";
    pub const PUBLISH: &'static str = "Broker.Publish";
    pub const PROCESS_GOL: &'static str = "Broker.ProcessGol";
    pub const COUNT_ALIVE: &'static str = "Broker.CountAliveCells";
    pub const QUIT: &'static str = "Broker.Quit";
    pub const SCREENSHOT: &'static str = "Broker.Screenshot";
    pub const PAUSE: &'static str = "Broker.Pause";
    pub const KILL: &'static str = "Broker.Kill";
}