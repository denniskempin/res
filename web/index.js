async function run() {
    let res = await import('../target/wasm-pack/res.js');

    let loading_div = document.getElementById("center_text");
    function init() {
        // We need to start the app during a user interaction so that audio
        // can be initialized.
        res.start_app("the_canvas_id");
        loading_div.remove();
    }
    loading_div.addEventListener("click", init);
}

run();
