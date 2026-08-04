# MicroRust

> **Focus on the domain and the use cases, not on the “glue code”**

MicroRust Application Framework

A modern rust application framework, asynchronous and with batteries included.

Inspired by frameworks like Spring Boot, Micronaut and Quarkus.

> Status: POC and WIP

## Features
* Dependency Injection

## Inbound Adapter (Server)
- REST / JSON
  - Topcoat
- HTML WebApp
  - Topcoat
- GraphQL
- gRPC
- Message Subscription
- MCP

## Outbound Adapter (Client)
- Relational Database
  - Toasty
- Document Database
- In-Memory Database
- Object Store
- REST / JSON
- GraphQL
- gRPC
- Message Publishing
- MCP
- IAM / IDM

## Examples

### Dependency Injection
```rust
use microrust::inject::{get_instance, singleton};

struct MyStruct {}

#[singleton(new)]
impl MyStruct {
    pub fn new() -> Self {
        println!("MyStruct::new() called");
        Self {}
    }

    async fn print(&self) {
        println!("Hello, MyStruct");
    }
}

#[tokio::main]
async fn main() {
    let inst = get_instance::<MyStruct>();
    // let inst: Arc<MyStruct> = get_instance();
    inst.print().await;
}
```

## Layers and Dependency Inversion

![Dependency Inversion](diagram.png)
