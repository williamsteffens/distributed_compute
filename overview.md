# problem formulation

something about sustainability

A general-purpose distributed GPU compute platform that explores the tradeoffs between native and browser-based workers, with a focus on workload scheduling, fault tolerance, verification, and extensibility.

# Specs? 

# Overview of report structure

# Pros and cons of native runtime
# pros and cons of browser based runtime


# research questions
How much overhead does distributed GPU execution introduce?

How does WebGPU compare with native GPU runtimes for different workloads?

How reliable are browser-based compute clients?

What scheduling strategies work well with heterogeneous GPUs?

How can results be verified without doubling computation cost?

How should a compute marketplace represent capacity, pricing, and trust?

WASM + SIMD

# MVP: distributed matrix multiplication

{
  "type": "matrix_multiply",
  "input": {
    "a": "...",
    "b": "..."
  },
  "parameters": {
    "m": 4096,
    "n": 4096,
    "k": 4096
  }
}

## Control plane

Owns jobs, scheduling, workers, and the event stream.

API server

Job submission, worker registration, status

Scheduler

Assigns work to suitable workers

Job store

Postgres or SQLite initially

Event system

WASM + SIMD

WebSocket or SSE initially

CAP theorem
can we do different things at different times?
so CA, and then AP? or have different systems for it to have each system do one pair?

Job latency
├── scheduling
├── network
├── data transfer
├── GPU execution
└── result transfer






# Run only the runtime benchmarks
cargo bench -p runtime --bench runtime

# Run a specific benchmark
cargo bench -p runtime --bench runtime -- vector_add

# Open the HTML report (after the run)






# Check that the crate compiles for WASM
cargo check --workspace  --target wasm32-unknown-unknown







cargo run -p worker

cargo run -p server start <--port portnumber>

cargo test -p runtime


need to open local for secure context need for crypto lib
npm run dev











curl -X POST http://localhost:42069/jobs \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "11111111-1111-1111-1111-111111111111",
    "workload": "hello",
    "parameters": {
      "message": "hello distributed world"
    }
  }'

curl -X POST http://localhost:42069/jobs \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "11111111-1111-1111-1111-111111111111",
    "workload": "hello",
    "parameters": {
      "message": "hello distributed world"
    }
  }'


i wanna return the id, not have the client give it to me...

i kinda wanna add validation of the payload at some point in the server
do i want services? or something else...










curl -X POST http://localhost:42069/jobs \
    -H 'Content-Type: application/json' \
    -d '{
        "workload": "vector_add",
        "parameters": {
            "a": [1,2,3,4],
            "b": [1,2,3,4]
        }
    }'

curl -X POST http://localhost:42069/jobs \
    -H 'Content-Type: application/json' \
    -d '{
        "workload": "vector_scale",
        "parameters": {
            "values": [1,2,3,4],
            "scalar": 10
        }
    }'


curl http://localhost:42069/jobs/<id>