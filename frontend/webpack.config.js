const path = require("path");
const TerserJsPlugin = require("terser-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const OptimizeCssAssetsPlugin = require("optimize-css-assets-webpack-plugin");

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
      {
        test: /\.css$/i,
        use: [MiniCssExtractPlugin.loader, "css-loader"],
      },
    ],
  },
  optimization: {
    moduleIds: "hashed",
    splitChunks: {
      chunks: "all",
      cacheGroups: {
        cards: {
          test: /([\\/]playing-cards(-4color)?[\\/])|(SvgCard.tsx)/,
          name(_) {
            return "playing-cards";
          },
        },
      },
    },
    minimizer: [
      new TerserJsPlugin({ sourceMap: true }),
      new OptimizeCssAssetsPlugin({}),
    ],
  },
  output: {
    filename: "[name].[contenthash].js",
  },
  performance: {
    hints: false,
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "shengji-wasm"),
      outName: "shengji-core",
    }),
    new HtmlWebpackPlugin({
      filename: "index.html",
      template: "static/index.html",
    }),
    new MiniCssExtractPlugin({
      filename: "style.css",
    }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: "static/rules.html",
          to: "rules.html",
        },
        {
          from: "static/timer-worker.js",
          to: "timer-worker.js",
        },
        {
          from: "static/434472_dersuperanton_taking-card.mp3",
          to: "434472_dersuperanton_taking-card.mp3",
        },
      ],
    }),
  ],
};
