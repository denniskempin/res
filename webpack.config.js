const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");
const target = path.resolve(__dirname, "target/wasm-pack");

module.exports = {
    mode: "production",
    entry: {
        index: "./web/index.js"
    },
    output: {
        path: dist,
        filename: "[name].js"
    },
    devServer: {
        contentBase: dist,
    },
    plugins: [
        new CopyPlugin([
            path.resolve(__dirname, "web/index.html")
        ]),

        new WasmPackPlugin({
            crateDirectory: __dirname,
            outDir: target,
            outName: "res"
        }),
    ]
};
