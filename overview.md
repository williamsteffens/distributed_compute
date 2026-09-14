# problem formulation

something about sustainability

A general-purpose distributed GPU compute platform that explores the tradeoffs between native and browser-based workers, with a focus on workload scheduling, fault tolerance, verification, and extensibility.

# Specs? 

# Overview of report structure

# Pros and cons of native runtime
# pros and cons of browser based runtime


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

WebSocket or SSE initially