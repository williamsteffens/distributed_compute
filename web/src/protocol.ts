export type Workload =
    | "VectorAdd"
    | "VectorScale"
    | "Saxpy"
    | "MatrixMultiply"
    | "Mandelbrot"
    | "MonteCarlo";

export interface WorkUnit {
    id: string; 
    job_id: string;
    workload: Workload;
    payload: string;
    assigned_worker: string | null;
    lease_expires_at: string | null;
    attempts: number;
}

export interface JobResult {
    work_unit_id: string; 
    success: boolean;
    result: unknown;
}

export interface WorkerRegistration {
    worker_id: string; 
    hostname: string;
}