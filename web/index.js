async function run() {
    let res = await import('../target/wasm-pack/res.js');
    res.start_app("the_canvas_id");
    document.getElementById("center_text").remove();
}

run();
