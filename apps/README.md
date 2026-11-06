# BBCC Applications

This directory contains the executable applications that make up the distributed computing framework.

## Applications

### Broker

The coordination component responsible for managing task distribution and communication between clients and providers.

[Read the Broker documentation](broker/README.md)

### Client

The application-facing component used to submit computational workloads and retrieve results.

[Read the Client documentation](client/README.md)

### Provider

The worker component responsible for executing computational workloads assigned by the broker.

[Read the Provider documentation](provider/README.md)

## Architecture

The applications work together to enable distributed task execution:

1. The client submits a workload to the broker.
2. The broker schedules the workload and assigns it to an available provider.
3. The provider executes the assigned workload.
4. Execution status and results are returned through the framework.

The exact communication flow depends on the protocol and implementation of each application.

## Development

```bash
cargo run -p broker start <--port-number>
cargo 
```

## Building

Build all workspace applications:

```bash
cargo build --workspace
```

Build an individual application:

```bash
cargo build -p broker
cargo build -p provider
```

## Configuration

Each application may have its own configuration, environment variables, and runtime requirements. Refer to its README for details.
