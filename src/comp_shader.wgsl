/*
    // Using an pipeline-overridable constant.
    @id(42) override block_width = 12u;
    @compute @workgroup_size(block_width)
    fn shuffler() { }
*/
@group(0) @binding(0) var<storage, read> inbuf: array<f32>;
@group(0) @binding(1) var<storage, read_write> outbuf: array<f32>;

@compute @workgroup_size(1) fn cs_main(
    @builtin(global_invocation_id) id: vec3u
) {
    let i = id.x;
    if i >= arrayLength(&outbuf) || i >= arrayLength(&inbuf) {
        return ;
    }
    outbuf[i] = inbuf[i] * inbuf[i];
}