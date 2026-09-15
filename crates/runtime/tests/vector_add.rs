use runtime::GpuRuntime;

#[test]
fn vector_add_works() {
    let runtime = GpuRuntime::new()
        .expect("failed to initialize GPU runtime");

    let a = vec![1.0, 2.0, 3.0, 4.0];
    let b = vec![10.0, 20.0, 30.0, 40.0];

    let result = runtime
        .vector_add(&a, &b)
        .expect("vector_add failed");

    assert_eq!(
        result,
        vec![11.0, 22.0, 33.0, 44.0]
    );
}