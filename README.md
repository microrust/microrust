# MicroRust

> **Focus on the domain and the use cases, not on the “glue code”**

MicroRust Application Framework

A modern rust application framework, asynchronous and with batteries included.

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

## Docs

- [Documentation](docs/README.md)
- https://docs.rs/microrust/latest/microrust/

## Layers and Dependency Inversion

You have two ways to take advantage of dependency injection:
- Dependency inversion using traits
- Dependency injection using structs

![Dependency Management](docs/diagram.png)
