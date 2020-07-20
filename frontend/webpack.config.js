const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  mode: "production",
  devtool: "source-map",
  resolve: {
    extensions: [".ts", ".tsx", ".js", ".jsx", ".svg"],
  },
  entry: "./src/index.tsx",
  module: {
    rules: [
      {
        test: /\.ts(x?)$/,
        exclude: /node_modules/,
        use: [
          {
            loader: "ts-loader",
          },
        ],
      },
    ],
  },
  optimization: {
    moduleIds: 'hashed',
    splitChunks: {
      chunks: "async",
      cacheGroups: {
        cards: {
          test: /([\\/]playing-cards(-4color)?[\\/])|(SvgCard.tsx)/,
          name(_) {
            return "playing-cards";
          },
        },
        vendors: {
          test: /[\\/]node_modules[\\/]/,
          name: "vendors",
        },
      },
    },
  },
  performance: {
    hints: false,
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "shengji-wasm"),
      outName: "shengji-core"
    })
  ]
};
