module.exports = {
  mode: "production",
  devtool: "source-map",
  resolve: {
    extensions: [".ts", ".tsx"]
  },
  module: {
    rules: [
      {
        test: /\.ts(x?)$/,
        exclude: /node_modules/,
        use: [
          {
            loader: "ts-loader"
          }
        ]
      },
    ]
  },
  externals: {
    react: "React",
    "react-dom": "ReactDOM"
  }
};
