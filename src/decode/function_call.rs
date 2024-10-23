pub struct FunctionCall {}

impl FunctionCall {
    //
    pub const PING: u8 = 1;
    //
    //

    pub const SUBSCRIBE: u8 = 4;
    pub const UNSUBSCRIBE: u8 = 5;
    //
    //

    pub const PROCESS_GOL: u8 = 8;
    pub const PROCESS_SLICE: u8 = 9;
    //
    //

    pub const PAUSE: u8 = 12;
    pub const SCREENSHOT: u8 = 13;
    pub const QUIT: u8 = 14;
    pub const KILL: u8 = 15;
}
