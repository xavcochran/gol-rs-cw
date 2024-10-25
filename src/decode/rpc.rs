use core::fmt;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::stubs::stubs::{BrokerRequest, BrokerResponse, StatusReport, Subscription};

// custom error type for rpc error handling
pub enum RpcError {
    Io(std::io::Error),
    Other(String),
    HandlerNotFound,
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcError::Io(e) => write!(f, "IO error: {}", e),
            RpcError::Other(msg) => write!(f, "Error: {}", msg),
            RpcError::HandlerNotFound => {
                write!(f, "Function handler not found in register handlers!")
            }
        }
    }
}
pub struct SubscriptionRpc;
pub struct BrokerRpc;
pub trait ValidRpcTypes {
    type Input: Send;
    type Output: Send;
}

impl ValidRpcTypes for SubscriptionRpc {
    type Input = Subscription;
    type Output = StatusReport;
}

impl ValidRpcTypes for BrokerRpc {
    type Input = BrokerRequest;
    type Output = BrokerResponse;
}

// chat gpt and claude sonnet helped with the generic type for async handling.
// All the input types needed to implement the send trait for thread safety and also needed to handle futures
#[async_trait::async_trait]
trait Handler<T: ValidRpcTypes + Send> {
    async fn call(&self, input: T::Input) -> Result<T::Output, RpcError>;
}
#[async_trait::async_trait]
impl<T, F, Fut> Handler<T> for F
where
    T: ValidRpcTypes + Send + 'static,
    F: for<'a> Fn(T::Input) -> Fut + Send + Sync,
    Fut: Future<Output = Result<T::Output, RpcError>> + Send,
{
    async fn call(&self, input: T::Input) -> Result<T::Output, RpcError> {
        self(input).await
    }
}

pub struct Rpc {
    handlers: HashMap<u8, Box<dyn Any + Send + Sync>>,
}

impl Rpc {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    // chat gpt also helped with types here
    /// adds a function to the handlers with the specified function id
    pub fn register<T, F, Fut>(&mut self, call: u8, handler: F)
    where
        T: ValidRpcTypes + Send + 'static,
        F: for<'a> Fn(T::Input) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T::Output, RpcError>> + Send + 'static,
    {
        let handler = Box::new(handler) as Box<dyn Handler<T> + Send + Sync>;
        self.handlers.insert(call, Box::new(handler));
    }

    /// Fetches function pointer from hashmap using function id.
    /// Returns rpc error with error if not found or with error from rpc call itself
    pub async fn dispatch<T: ValidRpcTypes + Send + 'static>(
        &self,
        call: u8,
        input: T::Input,
    ) -> Result<T::Output, RpcError> {
        self.handlers
            //
            // gets function from map
            .get(&call)
            //
            // checks that function is of expected type with correct input types e.g. if subscribe function is called it should only have Subscribtion and Status report types
            .and_then(|handler| handler.downcast_ref::<Box<dyn Handler<T> + Send + Sync>>())
            //
            // checks that function is valid
            .ok_or(RpcError::HandlerNotFound)?
            //
            // calls function
            .call(input)
            .await
    }
}
